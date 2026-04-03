use crate::api::client::DEFAULT_USER_AGENT;
use crate::error::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QRKeyResponseData {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Debug, Deserialize)]
struct QRKeyResponse {
    data: QRKeyResponseData,
}

#[derive(Debug, Deserialize)]
pub struct QrPollResponseData {
    pub url: String,
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct QRPollResponse {
    data: QrPollResponseData,
}

#[derive(Debug, Deserialize)]
struct NavResponse {
    data: NavData,
}

#[derive(Debug, Deserialize)]
struct NavData {
    mid: i64,
}

#[derive(Debug, Deserialize)]
struct RoomInfoResponse {
    data: RoomInfoData,
}

#[derive(Debug, Deserialize)]
struct RoomInfoData {
    roomid: i64,
}

pub fn generate_qr_code() -> Result<QRKeyResponseData> {
    let response =
        minreq::get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
            .with_header("User-Agent", DEFAULT_USER_AGENT)
            .send()?;

    let response_text = response.as_str()?;
    let qr_response: QRKeyResponse = serde_json::from_str(response_text)?;
    Ok(qr_response.data)
}

pub fn poll_qr_status(qrcode_key: &str) -> Result<QrPollResponseData> {
    let url = format!(
        "https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key={}",
        qrcode_key
    );
    let response = minreq::get(&url)
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .send()?;

    let response_text = response.as_str()?;
    let poll_response: QRPollResponse = serde_json::from_str(response_text)?;
    Ok(poll_response.data)
}

pub fn get_roomid(sessdata: &str) -> Result<i32> {
    let response = minreq::get("https://api.bilibili.com/x/web-interface/nav")
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .with_header("Cookie", format!("SESSDATA={}", sessdata))
        .send()?;

    let response_text = response.as_str()?;
    let nav_response: NavResponse = serde_json::from_str(response_text)?;
    let user_code = nav_response.data.mid.to_string();

    let url = format!(
        "https://api.live.bilibili.com/room/v1/Room/getRoomInfoOld?mid={}",
        user_code
    );
    let response = minreq::get(&url)
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .send()?;

    let response_text = response.as_str()?;
    let room_info: RoomInfoResponse = serde_json::from_str(response_text)?;
    Ok(room_info.data.roomid as i32)
}
