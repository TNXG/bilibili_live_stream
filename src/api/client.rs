use super::{Api, Endpoints, cookies::validate_credentials, response::Envelope};
use crate::error::{BiliLiveError, Result};
use crate::model::Credentials;
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::cookie::Jar;
use reqwest::{Url, header};
use serde::de::DeserializeOwned;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

const USER_AGENT: &str = "Mozilla/5.0 BiliLive/0.1";
const MAX_RESPONSE: u64 = 4 * 1024 * 1024;

impl Api {
    pub fn new() -> Result<Self> {
        Self::configured(
            Endpoints::from_environment()?,
            PathBuf::from("cookies.json"),
        )
    }

    /// Connect to explicitly configured local origins with an independent cookie store.
    /// All origins must be literal loopback HTTP(S) origins without credentials or paths.
    pub fn with_origins(
        api: &str,
        passport: &str,
        web: &str,
        cookie_path: impl Into<PathBuf>,
    ) -> Result<Self> {
        Self::configured(Endpoints::local(api, passport, web)?, cookie_path.into())
    }

    fn configured(endpoints: Endpoints, cookie_path: PathBuf) -> Result<Self> {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(Arc::clone(&jar))
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            // Login redirects are followed manually after validating each destination.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| BiliLiveError::Network("无法创建 HTTP 客户端".into()))?;
        Ok(Self {
            client,
            jar,
            endpoints,
            cookie_path,
        })
    }

    pub(super) fn request(&self, request: RequestBuilder) -> Result<Envelope> {
        let response = request
            .send()
            .map_err(|_| BiliLiveError::Network("网络连接失败或超时".into()))?;
        if !response.status().is_success() {
            return Err(BiliLiveError::Network(format!(
                "HTTP 状态 {}",
                response.status().as_u16()
            )));
        }
        let mut bytes = Vec::new();
        response
            .take(MAX_RESPONSE + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| BiliLiveError::Network("读取接口响应失败".into()))?;
        if bytes.len() as u64 > MAX_RESPONSE {
            return Err(BiliLiveError::Parse("接口响应过大".into()));
        }
        Envelope::parse(&bytes)
    }

    pub(super) fn data<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T> {
        self.request(request)?.decode()
    }

    pub(super) fn live_url(&self, path: &str) -> Url {
        self.endpoints.live.join(path).expect("static API path")
    }

    pub(super) fn authenticated(
        &self,
        request: RequestBuilder,
        c: &Credentials,
    ) -> Result<RequestBuilder> {
        validate_credentials(c)?;
        Ok(request
            .header(
                header::COOKIE,
                format!("SESSDATA={}; bili_jct={}", c.sessdata, c.csrf_token),
            )
            .header(header::REFERER, self.endpoints.web.as_str()))
    }

    pub(super) fn seed_cookies(&self, c: &Credentials) {
        for base in [
            &self.endpoints.live,
            &self.endpoints.nav,
            &self.endpoints.passport,
            &self.endpoints.web,
        ] {
            for (name, value) in [("SESSDATA", &c.sessdata), ("bili_jct", &c.csrf_token)] {
                let scope = if self.endpoints.fixture {
                    "Path=/"
                } else {
                    "Domain=.bilibili.com; Path=/; Secure"
                };
                self.jar
                    .add_cookie_str(&format!("{name}={value}; {scope}; HttpOnly"), base);
            }
        }
    }
}
