use crate::api::passport::get_roomid;
use crate::error::{BiliLiveError, Result};
use crate::user_success;
use crate::utils::string::get_query_string;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cookies {
    pub room_id: i32,
    pub sessdata: String,
    pub csrf_token: String,
}

pub fn save_cookies(set_cookies_url: &str) -> Result<()> {
    let bili_sessdata = get_query_string("SESSDATA", set_cookies_url);
    let csrf = get_query_string("bili_jct", set_cookies_url);
    let cookies = Cookies {
        room_id: get_roomid(&bili_sessdata)?,
        sessdata: bili_sessdata,
        csrf_token: csrf,
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
