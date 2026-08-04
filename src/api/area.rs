use crate::api::client::DEFAULT_USER_AGENT;
use crate::error::{BiliLiveError, Result};

pub fn fetch_area_list() -> Result<serde_json::Value> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("https://api.live.bilibili.com/room/v1/Area/getList")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .send()
        .map_err(|e| BiliLiveError::Network(e.to_string()))?;

    let area_list: serde_json::Value =
        serde_json::from_str(&resp.text().map_err(|e| BiliLiveError::Network(e.to_string()))?)?;
    Ok(area_list)
}
