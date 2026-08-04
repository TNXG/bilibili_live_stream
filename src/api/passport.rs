use crate::api::client::DEFAULT_USER_AGENT;
use crate::error::{BiliLiveError, Result};
use reqwest::cookie::CookieStore;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct QRKeyResponseData {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Debug, Deserialize)]
struct QRKeyResponse {
    data: QRKeyResponseData,
}

#[derive(Debug, Deserialize)]
pub struct QrPollResponseData {
    pub url: String,
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct QRPollResponse {
    data: QrPollResponseData,
}

/// 封装了 cookie jar 的登录 HTTP 客户端
pub struct LoginClient {
    pub client: reqwest::blocking::Client,
    jar: Arc<reqwest::cookie::Jar>,
}

impl LoginClient {
    pub fn new() -> Result<Self> {
        let jar = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::blocking::Client::builder()
            .cookie_provider(Arc::clone(&jar))
            .user_agent(DEFAULT_USER_AGENT)
            .build()
            .map_err(|e| BiliLiveError::Network(e.to_string()))?;
        Ok(Self { client, jar })
    }

    /// 生成登录二维码
    pub fn generate_qr_code(&self) -> Result<QRKeyResponseData> {
        let resp = self
            .client
            .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
            .send()
            .map_err(|e| BiliLiveError::Network(e.to_string()))?;
        let qr_response: QRKeyResponse =
            serde_json::from_str(&resp.text().map_err(|e| BiliLiveError::Network(e.to_string()))?)?;
        Ok(qr_response.data)
    }

    /// 轮询扫码状态；登录成功时会自动通过 cookie jar 保存 Set-Cookie
    pub fn poll_qr_status(&self, qrcode_key: &str) -> Result<QrPollResponseData> {
        let url = format!(
            "https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key={}",
            qrcode_key
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| BiliLiveError::Network(e.to_string()))?;
        let poll_response: QRPollResponse =
            serde_json::from_str(&resp.text().map_err(|e| BiliLiveError::Network(e.to_string()))?)?;
        Ok(poll_response.data)
    }

    /// 登录成功后，访问 crossDomain URL 完成 cookie 注入，然后从 jar 提取凭证
    pub fn finish_login(&self, cross_domain_url: &str) -> Result<(String, String)> {
        // 访问 crossDomain URL，服务器通过 Set-Cookie 下发 SESSDATA 等
        self.client
            .get(cross_domain_url)
            .send()
            .map_err(|e| BiliLiveError::Network(e.to_string()))?;

        // 从 cookie jar 提取 SESSDATA 和 bili_jct
        let bilibili_url: reqwest::Url = "https://bilibili.com"
            .parse()
            .expect("invalid bilibili URL");

        let cookies_raw = match self.jar.cookies(&bilibili_url) {
            Some(header) => header.to_str().unwrap_or("").to_string(),
            None => String::new(),
        };

        let sessdata = extract_cookie_value(&cookies_raw, "SESSDATA").unwrap_or_default();
        let bili_jct = extract_cookie_value(&cookies_raw, "bili_jct").unwrap_or_default();

        if sessdata.is_empty() {
            return Err(BiliLiveError::Api(
                "获取登录凭证(SESSDATA)失败".to_string()
            ));
        }
        Ok((sessdata, bili_jct))
    }
}

/// 从 cookie 头字符串中提取指定 cookie 的值
fn extract_cookie_value(cookie_str: &str, name: &str) -> Option<String> {
    let prefix = format!("{}=", name);
    for part in cookie_str.split(';') {
        let trimmed = part.trim();
        if let Some(value) = trimmed.strip_prefix(&prefix) {
            return Some(value.to_string());
        }
    }
    None
}

/// 调用 nav 接口获取用户 mid，再通过直播接口获取房间号
pub fn get_roomid(sessdata: &str) -> Result<i32> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .map_err(|e| BiliLiveError::Network(e.to_string()))?;

    let nav_json: serde_json::Value =
        serde_json::from_str(&resp.text().map_err(|e| BiliLiveError::Network(e.to_string()))?)?;

    let code = nav_json["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        return Err(BiliLiveError::Api(format!(
            "获取用户信息失败 (code={}): {}",
            code,
            nav_json["message"].as_str().unwrap_or("未知错误")
        )));
    }

    let is_login = nav_json["data"]["isLogin"].as_bool().unwrap_or(false);
    if !is_login {
        return Err(BiliLiveError::Api(
            "当前登录状态已失效，请重新登录".to_string(),
        ));
    }

    let mid = nav_json["data"]["mid"]
        .as_i64()
        .ok_or_else(|| BiliLiveError::Parse("获取用户信息失败 (缺少 mid)".to_string()))?;

    let url = format!(
        "https://api.live.bilibili.com/room/v1/Room/getRoomInfoOld?mid={}",
        mid
    );
    let resp = client
        .get(&url)
        .header("User-Agent", DEFAULT_USER_AGENT)
        .send()
        .map_err(|e| BiliLiveError::Network(e.to_string()))?;

    let room_info: RoomInfoResponse =
        serde_json::from_str(&resp.text().map_err(|e| BiliLiveError::Network(e.to_string()))?)?;
    Ok(room_info.data.roomid as i32)
}

#[derive(Debug, Deserialize)]
struct RoomInfoResponse {
    data: RoomInfoData,
}

#[derive(Debug, Deserialize)]
struct RoomInfoData {
    roomid: i64,
}

#[cfg(test)]
mod tests {
    use super::extract_cookie_value;

    #[test]
    fn test_extract_sessdata_from_cookie_header() {
        assert_eq!(
            extract_cookie_value(
                "SESSDATA=abc123; bili_jct=def456; DedeUserID=789",
                "SESSDATA"
            ),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_extract_bili_jct_from_cookie_header() {
        assert_eq!(
            extract_cookie_value(
                "SESSDATA=abc123; bili_jct=def456; DedeUserID=789",
                "bili_jct"
            ),
            Some("def456".to_string())
        );
    }

    #[test]
    fn test_extract_missing_cookie_returns_none() {
        assert_eq!(
            extract_cookie_value("DedeUserID=789", "SESSDATA"),
            None
        );
    }

    #[test]
    fn test_extract_from_empty_header() {
        assert_eq!(extract_cookie_value("", "SESSDATA"), None);
    }
}
