use crate::api::client::DEFAULT_USER_AGENT;
use crate::auth::cookies::read_cookies;
use crate::error::{BiliLiveError, Result};
use crate::{user_info, user_warning};

pub fn check_status() -> Result<bool> {
    user_info!("检查登录状态...");
    if !std::path::Path::new("cookies.json").exists() {
        user_warning!("cookies.json文件不存在");
        return Ok(false);
    }
    if std::fs::read_to_string("cookies.json")
        .map_err(BiliLiveError::Io)?
        .is_empty()
    {
        user_warning!("cookies.json文件为空");
        return Ok(false);
    }
    let sessdata = read_cookies()?.sessdata;
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .map_err(|e| BiliLiveError::Network(e.to_string()))?;

    let response_json: serde_json::Value =
        serde_json::from_str(&resp.text().map_err(|e| BiliLiveError::Network(e.to_string()))?)?;

    let is_login = response_json["data"]["isLogin"].as_bool().unwrap_or(false);
    if is_login {
        Ok(true)
    } else {
        user_warning!("当前登录状态已失效，请重新登录");
        Ok(false)
    }
}
