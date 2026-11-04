use crate::error::{BiliLiveError, Result};
use reqwest::Url;

pub(super) struct Endpoints {
    pub live: Url,
    pub nav: Url,
    pub passport: Url,
    pub web: Url,
    pub fixture: bool,
}

impl Endpoints {
    pub fn production() -> Self {
        Self {
            live: Url::parse("https://api.live.bilibili.com/").expect("static URL"),
            nav: Url::parse("https://api.bilibili.com/").expect("static URL"),
            passport: Url::parse("https://passport.bilibili.com/").expect("static URL"),
            web: Url::parse("https://www.bilibili.com/").expect("static URL"),
            fixture: false,
        }
    }

    pub fn from_environment() -> Result<Self> {
        let names = ["BILI_API_BASE", "BILI_PASSPORT_BASE", "BILI_WEB_BASE"];
        let mut values = [None, None, None];
        for (i, name) in names.iter().enumerate() {
            values[i] = match std::env::var(name) {
                Ok(value) => Some(value),
                Err(std::env::VarError::NotPresent) => None,
                Err(_) => return Err(BiliLiveError::Parse("本地接口覆盖包含无效文本".into())),
            };
        }
        match values {
            [None, None, None] => Ok(Self::production()),
            [Some(api), Some(passport), Some(web)] => Self::local(&api, &passport, &web),
            _ => Err(BiliLiveError::Parse(
                "本地接口覆盖必须同时设置全部三个 BILI_*_BASE".into(),
            )),
        }
    }

    pub fn local(api: &str, passport: &str, web: &str) -> Result<Self> {
        let live = loopback_origin(api)?;
        Ok(Self {
            nav: live.clone(),
            live,
            passport: loopback_origin(passport)?,
            web: loopback_origin(web)?,
            fixture: true,
        })
    }

    pub fn allows_login_url(&self, url: &Url) -> bool {
        if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
            return false;
        }
        if self.fixture {
            return [&self.live, &self.passport, &self.web]
                .iter()
                .any(|base| base.origin() == url.origin());
        }
        if url.scheme() != "https" || url.port_or_known_default() != Some(443) {
            return false;
        }
        // The QR poll cross-domain endpoint is served from passport.bilibili.com
        // and, for game accounts, passport.biligame.com.
        url.host_str().is_some_and(super::login_host_allowed)
    }
}

fn loopback_origin(raw: &str) -> Result<Url> {
    let invalid =
        || BiliLiveError::Parse("本地接口覆盖仅允许无凭据的字面量回环 HTTP(S) origin".into());
    let url = Url::parse(raw).map_err(|_| invalid())?;
    // URL parsing canonicalizes shortened and decimal IPv4; require the original literal.
    let authority = raw
        .split_once("://")
        .map_or("", |(_, rest)| rest.split('/').next().unwrap_or(""));
    let literal = if authority.starts_with('[') {
        authority.split_once(']').map_or("", |(host, _)| &host[1..])
    } else {
        authority.split(':').next().unwrap_or("")
    };
    let loopback = literal
        .parse::<std::net::IpAddr>()
        .is_ok_and(|ip| ip.is_loopback());
    if !loopback
        || raw.chars().any(char::is_whitespace)
        || !matches!(url.scheme(), "http" | "https")
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(invalid());
    }
    Ok(url)
}
