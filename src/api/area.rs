use crate::api::client::DEFAULT_USER_AGENT;
use crate::error::Result;

pub fn fetch_area_list() -> Result<serde_json::Value> {
    let response = minreq::get("https://api.live.bilibili.com/room/v1/Area/getList")
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .send()?;

    let response_text = response.as_str()?;
    let area_list: serde_json::Value = serde_json::from_str(response_text)?;
    Ok(area_list)
}
