use crate::api::client::DEFAULT_USER_AGENT;
use crate::auth::cookies::read_cookies;
use crate::error::{BiliLiveError, Result};
use crate::live::stats::get_live_info;
use crate::utils::clipboard::copy_to_clipboard;
use crate::utils::string::mask_rtmp_code;
use crate::{user_success, user_warning};

pub fn start_live(area_id: &str, show_full_code: bool) -> Result<u64> {
    let cookies = read_cookies()?;

    let form_data = format!(
        "room_id={}&area_v2={}&csrf={}&platform=pc_link",
        cookies.room_id, area_id, cookies.csrf_token
    );

    let response = minreq::post("https://api.live.bilibili.com/room/v1/Room/startLive")
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .with_header("Content-Type", "application/x-www-form-urlencoded")
        .with_header("Cookie", format!("SESSDATA={}", cookies.sessdata))
        .with_header("platform", "web_electron_link")
        .with_body(form_data)
        .send()?;

    let response_text = response.as_str()?;
    let res: serde_json::Value = serde_json::from_str(response_text)?;

    if res["code"].as_i64() != Some(0) {
        return Err(BiliLiveError::Api(format!(
            "API返回错误: {}",
            res["message"].as_str().unwrap_or("未知错误")
        )));
    }

    let rtmp_addr = res["data"]["rtmp"]["addr"]
        .as_str()
        .ok_or_else(|| BiliLiveError::Parse("缺少rtmp地址".to_string()))?;
    let rtmp_code = res["data"]["rtmp"]["code"]
        .as_str()
        .ok_or_else(|| BiliLiveError::Parse("缺少rtmp code".to_string()))?;
    let live_key = res["data"]["live_key"]
        .as_str()
        .ok_or_else(|| BiliLiveError::Parse("缺少live_key".to_string()))?;

    user_success!("RTMP地址: {}", rtmp_addr);

    if show_full_code {
        user_success!("推流码: {}", rtmp_code);
    } else {
        user_success!("推流码: {}", mask_rtmp_code(rtmp_code));
    }

    match copy_to_clipboard(rtmp_code) {
        Ok(()) => user_success!("推流码已自动复制到剪贴板！"),
        Err(e) => user_warning!("复制到剪贴板失败: {}，请手动复制", e),
    }

    Ok(live_key.parse::<u64>()?)
}

pub fn stop_live(live_id: u64) -> Result<()> {
    let cookies = read_cookies()?;

    let form_data = format!(
        "room_id={}&csrf={}&platform=web_electron_link",
        cookies.room_id, cookies.csrf_token
    );

    let response = minreq::post("https://api.live.bilibili.com/room/v1/Room/stopLive")
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .with_header("Content-Type", "application/x-www-form-urlencoded")
        .with_header("Cookie", format!("SESSDATA={}", cookies.sessdata))
        .with_body(form_data)
        .send()?;

    let response_text = response.as_str()?;
    let res: serde_json::Value = serde_json::from_str(response_text)?;

    if res["code"].as_i64() != Some(0) {
        return Err(BiliLiveError::Api(format!(
            "API返回错误: {}",
            res["message"].as_str().unwrap_or("未知错误")
        )));
    }

    user_success!("成功关闭直播");

    get_live_info(live_id)?;

    Ok(())
}
