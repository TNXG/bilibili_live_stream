use crate::api::client::DEFAULT_USER_AGENT;
use crate::auth::cookies::read_cookies;
use crate::error::{BiliLiveError, Result};
use crate::user_info;

pub fn get_live_info(live_id: u64) -> Result<()> {
    let cookies = read_cookies()?;
    let url = format!(
        "https://api.live.bilibili.com/xlive/app-blink/v1/live/StopLiveData?live_key={}",
        live_id
    );

    let response = minreq::get(&url)
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .with_header("Content-Type", "application/json, text/plain, */*")
        .with_header("Cookie", format!("SESSDATA={}", cookies.sessdata))
        .send()?;

    let response_text = response.as_str()?;
    let res: serde_json::Value = serde_json::from_str(response_text)?;

    if res["code"].as_i64() != Some(0) {
        return Err(BiliLiveError::Api(format!(
            "API返回错误: {}",
            res["message"].as_str().unwrap_or("未知错误")
        )));
    }

    let data = &res["data"];
    user_info!("直播统计信息:");
    user_info!("新增粉丝 : {}", data["AddFans"].as_i64().unwrap_or(0));
    user_info!("弹幕数 : {}", data["DanmuNum"].as_i64().unwrap_or(0));
    user_info!("金仓鼠流水 : {}", data["HamsterRmb"].as_i64().unwrap_or(0));
    user_info!("直播时长 : {}", data["LiveTime"].as_i64().unwrap_or(0));
    user_info!("最大在线 : {}", data["MaxOnline"].as_i64().unwrap_or(0));
    user_info!(
        "新增粉丝勋章 : {}",
        data["NewFansClub"].as_i64().unwrap_or(0)
    );
    user_info!("累计观看 : {}", data["WatchedCount"].as_i64().unwrap_or(0));

    Ok(())
}
