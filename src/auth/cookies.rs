use crate::api::passport::get_roomid;
use crate::error::{BiliLiveError, Result};
use crate::user_success;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cookies {
    pub room_id: i32,
    pub sessdata: String,
    pub csrf_token: String,
}

/// 保存登录凭证（SESSDATA 和 csrf 已由 LoginClient 从 cookie jar 提取）
pub fn save_cookies(sessdata: &str, csrf_token: &str) -> Result<()> {
    let cookies = Cookies {
        room_id: get_roomid(sessdata)?,
        sessdata: sessdata.to_string(),
        csrf_token: csrf_token.to_string(),
    };

    let cookies_json = serde_json::to_string_pretty(&cookies)?;
    fs::write("cookies.json", cookies_json)?;
    user_success!("Cookies保存成功");
    Ok(())
}

pub fn read_cookies() -> Result<Cookies> {
    let cookies_str = std::fs::read_to_string("./cookies.json").map_err(BiliLiveError::Io)?;
    let cookies: Cookies = serde_json::from_str(&cookies_str).map_err(BiliLiveError::Json)?;
    Ok(cookies)
}
