use super::{
    Api,
    cookies::{cookie_value, persist_credentials, validate_credentials},
};
use crate::error::{BiliLiveError, Result};
use crate::model::{Account, Credentials, QrCodeData, QrPoll, ids::deserialize_id};
use reqwest::cookie::CookieStore;
use reqwest::{Url, header};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

#[derive(Deserialize)]
struct NavData {
    #[serde(rename = "isLogin")]
    is_login: bool,
    #[serde(default)]
    mid: Value,
    #[serde(default)]
    uname: String,
}

#[derive(Deserialize)]
struct RoomData {
    #[serde(deserialize_with = "deserialize_id")]
    roomid: u64,
}

impl Api {
    fn room_for_session(
        &self,
        sessdata: &str,
        csrf_token: &str,
    ) -> Result<Option<(Credentials, Account)>> {
        let provisional = Credentials {
            room_id: 1,
            sessdata: sessdata.into(),
            csrf_token: csrf_token.into(),
        };
        let nav = self.request(
            self.authenticated(
                self.client.get(
                    self.endpoints
                        .nav
                        .join("/x/web-interface/nav")
                        .expect("static API path"),
                ),
                &provisional,
            )?,
        )?;
        if nav.code == -101 {
            return Ok(None);
        }
        let nav: NavData = nav.decode()?;
        if !nav.is_login {
            return Ok(None);
        }
        let mid = nav
            .mid
            .as_u64()
            .or_else(|| nav.mid.as_str().and_then(|s| s.parse().ok()))
            .filter(|mid| *mid > 0)
            .ok_or_else(|| BiliLiveError::Parse("用户信息缺少有效 mid".into()))?;
        let room: RoomData = self.data(
            self.authenticated(
                self.client
                    .get(self.live_url("/room/v1/Room/getRoomInfoOld"))
                    .query(&[("mid", mid)]),
                &provisional,
            )?,
        )?;
        let credentials = Credentials {
            room_id: room.roomid,
            ..provisional
        };
        validate_credentials(&credentials)?;
        let account = Account {
            mid,
            name: nav.uname,
        };
        Ok(Some((credentials, account)))
    }

    pub fn check_session(&self) -> Result<Option<(Credentials, Account)>> {
        let bytes = match fs::read(&self.cookie_path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(BiliLiveError::Io(error)),
        };
        let c: Credentials = serde_json::from_slice(&bytes)
            .map_err(|_| BiliLiveError::Parse("cookies.json 格式损坏".into()))?;
        validate_credentials(&c)?;
        self.seed_cookies(&c);
        self.room_for_session(&c.sessdata, &c.csrf_token)
    }

    pub fn generate_qr(&self) -> Result<QrCodeData> {
        let qr: QrCodeData = self.data(
            self.client.get(
                self.endpoints
                    .passport
                    .join("/x/passport-login/web/qrcode/generate")
                    .expect("static API path"),
            ),
        )?;
        if qr.url.is_empty() || qr.qrcode_key.is_empty() {
            return Err(BiliLiveError::Parse("二维码响应缺少 URL 或 key".into()));
        }
        Ok(qr)
    }

    pub fn poll_qr(&self, key: &str) -> Result<QrPoll> {
        self.data(
            self.client
                .get(
                    self.endpoints
                        .passport
                        .join("/x/passport-login/web/qrcode/poll")
                        .expect("static API path"),
                )
                .query(&[("qrcode_key", key)]),
        )
    }

    pub fn finish_login(&self, raw_url: &str) -> Result<(Credentials, Account)> {
        let mut url =
            Url::parse(raw_url).map_err(|_| BiliLiveError::Parse("登录跳转地址无效".into()))?;
        for redirect in 0..=5 {
            if !self.endpoints.allows_login_url(&url) {
                return Err(BiliLiveError::Parse("拒绝不安全的登录跳转地址".into()));
            }
            let response = self
                .client
                .get(url.clone())
                .send()
                .map_err(|_| BiliLiveError::Network("登录跳转请求失败或超时".into()))?;
            if response.status().is_redirection() {
                if redirect == 5 {
                    return Err(BiliLiveError::Api("登录跳转次数过多".into()));
                }
                let location = response
                    .headers()
                    .get(header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| BiliLiveError::Parse("登录跳转缺少有效 Location".into()))?;
                url = url
                    .join(location)
                    .map_err(|_| BiliLiveError::Parse("登录跳转地址无效".into()))?;
                continue;
            }
            if !response.status().is_success() {
                return Err(BiliLiveError::Network(format!(
                    "登录跳转 HTTP 状态 {}",
                    response.status().as_u16()
                )));
            }
            let mut sessdata = None;
            let mut csrf_token = None;
            // The cross-domain endpoint may scope cookies to another B站 host,
            // so read them from this response before consulting the jar.
            for cookie in response.cookies() {
                match cookie.name() {
                    "SESSDATA" if sessdata.is_none() => {
                        sessdata = Some(cookie.value().to_owned());
                    }
                    "bili_jct" if csrf_token.is_none() => {
                        csrf_token = Some(cookie.value().to_owned());
                    }
                    _ => {}
                }
            }
            for base in [
                &self.endpoints.live,
                &self.endpoints.nav,
                &self.endpoints.web,
                &self.endpoints.passport,
                &url,
            ] {
                if let Some(cookies) = self.jar.cookies(base)
                    && let Ok(raw) = cookies.to_str()
                {
                    if sessdata.is_none() {
                        sessdata = cookie_value(raw, "SESSDATA");
                    }
                    if csrf_token.is_none() {
                        csrf_token = cookie_value(raw, "bili_jct");
                    }
                }
            }
            let sessdata =
                sessdata.ok_or_else(|| BiliLiveError::Api("登录未下发 SESSDATA".into()))?;
            let csrf_token =
                csrf_token.ok_or_else(|| BiliLiveError::Api("登录未下发 CSRF 凭据".into()))?;
            let (c, account) = self
                .room_for_session(&sessdata, &csrf_token)?
                .ok_or_else(|| BiliLiveError::Api("扫码登录凭据无效".into()))?;
            self.seed_cookies(&c);
            persist_credentials(&c, &self.cookie_path)?;
            return Ok((c, account));
        }
        unreachable!("bounded redirects return on final iteration")
    }
}
