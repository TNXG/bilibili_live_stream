use crate::api::client::DEFAULT_USER_AGENT;
use crate::auth::cookies::read_cookies;
use crate::error::{BiliLiveError, Result};

pub fn get_recent_live() -> Result<(String, String)> {
    let room_id = read_cookies()?.room_id;
    let url = format!(
        "https://api.live.bilibili.com/room/v1/Area/getMyChooseArea?roomid={}",
        room_id
    );

    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", DEFAULT_USER_AGENT)
        .send()
        .map_err(|e| BiliLiveError::Network(e.to_string()))?;

    let json: serde_json::Value =
        serde_json::from_str(&resp.text().map_err(|e| BiliLiveError::Network(e.to_string()))?)?;
    let data = &json["data"][0];
    let id = data["id"]
        .as_str()
        .ok_or_else(|| BiliLiveError::Parse("无法解析分区ID".to_string()))?
        .to_string();
    let name = data["name"]
        .as_str()
        .ok_or_else(|| BiliLiveError::Parse("无法解析分区名称".to_string()))?
        .to_string();
    Ok((id, name))
}
