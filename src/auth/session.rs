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
    let response = minreq::get("https://api.bilibili.com/x/web-interface/nav")
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .with_header("Cookie", format!("SESSDATA={}", sessdata))
        .send()?;

    let response_text = response.as_str()?;
    let response_json: serde_json::Value = serde_json::from_str(response_text)?;
    let code = response_json["code"]
        .as_i64()
        .ok_or_else(|| BiliLiveError::Parse("无法解析响应码".to_string()))?;
    if code == 0 {
        Ok(true)
    } else {
        user_warning!("登录状态异常");
        Ok(false)
    }
}
