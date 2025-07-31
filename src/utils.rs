use crate::error::{BiliLiveError, Result};
use crate::{user_info, user_input_prompt, user_success, user_warning};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use reqwest::header::{CONTENT_TYPE, COOKIE};
use serde::{Deserialize, Serialize};
use std::{fs, sync::LazyLock};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cookies {
    pub room_id: i32,
    pub sessdata: String,
    pub csrf_token: String,
}

#[derive(Debug, Deserialize)]
struct QRKeyResponseData {
    url: Url,
    qrcode_key: String,
}

#[derive(Debug, Deserialize)]
struct QRKeyResponse {
    data: QRKeyResponseData,
}

#[derive(Debug, Deserialize)]
struct QrPollResponseData {
    #[serde(deserialize_with = "deserialize_qr_poll_url")]
    url: Option<Url>,
    code: i32,
    #[allow(dead_code)]
    message: String,
}

fn deserialize_qr_poll_url<'de, D>(deserializer: D) -> std::result::Result<Option<Url>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let url_str: String = Deserialize::deserialize(deserializer)?;
    let Ok(url) = Url::parse(&url_str) else {
        return Ok(None);
    };
    Ok(Some(url))
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

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0";

static CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
    reqwest::blocking::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .build()
        .expect("Failed to create HTTP client")
});

#[derive(FromPrimitive)]
enum QrStatus {
    Waiting = 86101, // 等待扫码
    Scanned = 86090, // 已扫码，等待确认
    Success = 0,     // 登录成功
}

fn generate_qr_code() -> Result<QRKeyResponseData> {
    let response = CLIENT
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
        .send()?;

    let qr_response: QRKeyResponse = response.json()?;
    Ok(qr_response.data)
}

fn poll_qr_status(qrcode_key: &str) -> Result<QrPollResponseData> {
    let url = format!(
        "https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key={}",
        qrcode_key
    );
    let response = CLIENT.get(&url).send()?;
    let poll_response: QRPollResponse = response.json()?;
    Ok(poll_response.data)
}

pub fn get_query_string(name: &str, url: &Url) -> String {
    url.query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
        .unwrap_or_default()
}

pub fn get_roomid(sessdata: &str) -> Result<i32> {
    let response = CLIENT
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header(COOKIE, &format!("SESSDATA={}", sessdata))
        .send()?;

    let nav_response: NavResponse = response.json()?;
    let user_code = nav_response.data.mid.to_string();

    let url = format!(
        "https://api.live.bilibili.com/room/v1/Room/getRoomInfoOld?mid={}",
        user_code
    );
    let response = CLIENT.get(&url).send()?;

    let room_info: RoomInfoResponse = response.json()?;
    Ok(room_info.data.roomid as i32)
}

pub fn save_cookies(set_cookies_url: &Url) -> Result<()> {
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
    let reader = std::fs::File::open("./cookies.json")?;
    let cookies: Cookies = serde_json::from_reader(reader)?;
    Ok(cookies)
}

pub fn check_status() -> Result<bool> {
    user_info!("检查登录状态...");
    // 先检查一下文件是否存在
    if !std::path::Path::new("cookies.json")
        .try_exists()
        .unwrap_or_default()
    {
        user_warning!("cookies.json文件不存在");
        return Ok(false);
    }
    // 读取cookies.json文件
    let sessdata = read_cookies()?.sessdata;
    // 发送请求
    let response = CLIENT
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header(COOKIE, &format!("SESSDATA={}", sessdata))
        .send()?;

    // 解析响应
    let response_json: serde_json::Value = response.json()?;
    let code = response_json["code"]
        .as_i64()
        .ok_or_else(|| BiliLiveError::ParseError("无法解析响应码".to_string()))?;
    if code == 0 {
        return Ok(true);
    } else {
        user_warning!("登录状态异常");
        return Ok(false);
    }
}

pub fn get_area_choice() -> Result<u32> {
    let response = CLIENT
        .get("https://api.live.bilibili.com/room/v1/Area/getList")
        .send()?;

    let area_list: serde_json::Value = response.json()?;

    let Some(data) = area_list["data"].as_array() else {
        return Err(BiliLiveError::ParseError("无法解析分区列表".to_string()));
    };

    loop {
        // 显示一级分区
        user_info!("一级分区列表:");

        for (i, area) in data.iter().enumerate() {
            user_info!("{}. {}", i + 1, area["name"]);
        }

        user_input_prompt!("请输入一级分区编号: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let first_choice: usize = input.trim().parse()?;

        if first_choice == 0 {
            user_warning!("你是抱着多大的觉悟在一级菜单按下0的？");
            continue;
        }

        let Some(data) = area_list["data"].as_array() else {
            continue;
        };
        let Some(selected_first_area) = data.get(first_choice - 1) else {
            user_warning!("无效的选择，请重新输入");
            continue;
        };
        let Some(second_list) = selected_first_area["list"].as_array() else {
            user_warning!("无效的选择，请重新输入");
            continue;
        };
        loop {
            // 显示二级分区
            user_info!("二级分区列表 ({}):", selected_first_area["name"]);
            for (i, area) in second_list.iter().enumerate() {
                user_info!("{}. {} - {}", i + 1, area["name"], area["id"]);
            }

            user_input_prompt!("请输入二级分区编号(输入0返回): ");
            let mut second_input = String::new();
            std::io::stdin().read_line(&mut second_input)?;
            let second_choice: usize = second_input.trim().parse()?;

            if second_choice == 0 {
                user_warning!("你是抱着多大的觉悟在二级菜单按下0的？");
                break;
            }
            let Some(selected_area) = second_list.get(second_choice - 1) else {
                user_warning!("无效的选择，请重新输入");
                continue;
            };
            user_success!(
                "已选择分区: {} (ID: {})",
                selected_area["name"],
                selected_area["id"]
            );
            let id_str = selected_area["id"].as_str().unwrap_or_default();
            let id_str = id_str
                .chars()
                .filter(|c| c.is_numeric())
                .collect::<String>();
            return Ok(id_str.parse::<u32>()?);
        }

        user_warning!("无效的选择，请重新输入");
    }
}

pub fn start_login() -> Result<()> {
    user_info!("开始B站二维码登录流程...");

    let qr_data = generate_qr_code()?;
    user_info!("请使用B站手机客户端如下链接：{}", qr_data.url);

    user_info!("或使用B站手机客户端扫描如下二维码");

    print_qrcode_in_terminal(&qr_data.url)?;

    // 生成二维码图片并保存到本地
    generate_and_save_qrcode(&qr_data.url, "qrcode.png")?;
    user_success!("二维码已保存到 qrcode.png");

    user_info!("等待用户处理...");

    // 轮询扫码状态
    loop {
        let poll_data = poll_qr_status(&qr_data.qrcode_key)?;

        match QrStatus::from_i32(poll_data.code) {
            Some(QrStatus::Waiting) => {
                // 可以添加等待提示
            }
            Some(QrStatus::Scanned) => {
                user_info!("已处理，请在手机上确认登录");
            }
            Some(QrStatus::Success) => {
                user_success!("登录成功！");
                let url = poll_data.url.ok_or_else(|| {
                    BiliLiveError::ParseError("二维码登录成功后未返回URL".to_string())
                })?;
                save_cookies(&url)?;
                std::fs::remove_file("qrcode.png")?;
                break;
            }
            _ => {
                user_warning!(
                    "未知状态码: {}, 消息: {}",
                    poll_data.code,
                    poll_data.message
                );
                std::fs::remove_file("qrcode.png")?;
                return Err(BiliLiveError::ParseError(format!(
                    "二维码登录失败，状态码: {}, 消息: {}",
                    poll_data.code, poll_data.message
                )));
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    Ok(())
}

/// 生成二维码图片并保存到文件
fn generate_and_save_qrcode(url: &Url, filename: &str) -> Result<()> {
    use image::Luma;
    use qrcode::QrCode;
    use std::path::Path;

    // 生成二维码
    let code = QrCode::new(url.as_str().as_bytes())?;

    // 转换为图像
    let image = code
        .render::<Luma<u8>>()
        .quiet_zone(false) // 禁用静区（可选）
        .min_dimensions(200, 200) // 最小尺寸
        .build();

    // 保存为PNG文件
    let path = Path::new(filename);
    image.save(path)?;

    Ok(())
}

fn print_qrcode_in_terminal(url: &Url) -> Result<()> {
    use qrcode::QrCode;
    let code = QrCode::new(url.as_str().as_bytes())?;

    // 转换为ASCII字符串
    let string = code
        .render()
        .light_color("  ") // 浅色部分用空格
        .dark_color("██") // 深色部分用方块
        .quiet_zone(false)
        .build();

    user_info!("\n{}", string);
    Ok(())
}

// 获取用户最近直播过的分区信息
pub fn get_recent_live() -> Result<(String, String)> {
    let room_id = read_cookies()?.room_id;
    let url = format!(
        "https://api.live.bilibili.com/room/v1/Area/getMyChooseArea?roomid={}",
        room_id
    );
    let response = CLIENT.get(&url).send()?;

    let json: serde_json::Value = response.json()?;
    let data = &json["data"][0];
    let id = data["id"]
        .as_str()
        .ok_or_else(|| BiliLiveError::ParseError("无法解析分区ID".to_string()))?
        .to_string();
    let name = data["name"]
        .as_str()
        .ok_or_else(|| BiliLiveError::ParseError("无法解析分区名称".to_string()))?
        .to_string();
    Ok((id, name))
}

// 开始直播，获取推流码和推流地址
pub fn start_live(area_id: &str) -> Result<u64> {
    let cookies = read_cookies()?;

    // 构建表单数据
    let form_data = format!(
        "room_id={}&area_v2={}&csrf={}&platform=pc_link",
        cookies.room_id, area_id, cookies.csrf_token
    );

    let response = CLIENT
        .post("https://api.live.bilibili.com/room/v1/Room/startLive")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(COOKIE, &format!("SESSDATA={}", cookies.sessdata))
        .header("platform", "web_electron_link")
        .body(form_data)
        .send()?;

    let res = response.json::<serde_json::Value>()?;

    if res["code"].as_i64() != Some(0) {
        return Err(BiliLiveError::ApiError(format!(
            "API返回错误: {}",
            res["message"].as_str().unwrap_or("未知错误")
        )));
    }

    let rtmp_addr = res["data"]["rtmp"]["addr"]
        .as_str()
        .ok_or_else(|| BiliLiveError::ParseError("缺少rtmp地址".to_string()))?;
    let rtmp_code = res["data"]["rtmp"]["code"]
        .as_str()
        .ok_or_else(|| BiliLiveError::ParseError("缺少rtmp code".to_string()))?;
    let live_key = res["data"]["live_key"]
        .as_str()
        .ok_or_else(|| BiliLiveError::ParseError("缺少live_key".to_string()))?;

    user_success!("RTMP地址: {}", rtmp_addr);
    user_success!("推流码: {}", rtmp_code);

    Ok(live_key.parse::<u64>()?)
}

pub fn stop_live(live_id: u64) -> Result<()> {
    let cookies = read_cookies()?;

    // 构建表单数据
    let form_data = format!(
        "room_id={}&csrf={}&platform=web_electron_link",
        cookies.room_id, cookies.csrf_token
    );

    let response = CLIENT
        .post("https://api.live.bilibili.com/room/v1/Room/stopLive")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(COOKIE, &format!("SESSDATA={}", cookies.sessdata))
        .body(form_data)
        .send()?;

    let res: serde_json::Value = response.json()?;

    if res["code"].as_i64() != Some(0) {
        return Err(BiliLiveError::ApiError(format!(
            "API返回错误: {}",
            res["message"].as_str().unwrap_or("未知错误")
        )));
    }

    user_success!("成功关闭直播");

    get_live_info(live_id)?;

    Ok(())
}

fn get_live_info(live_id: u64) -> Result<()> {
    let cookies = read_cookies()?;
    let url = format!(
        "https://api.live.bilibili.com/xlive/app-blink/v1/live/StopLiveData?live_key={}",
        live_id
    );

    let response = CLIENT
        .get(&url)
        .header(CONTENT_TYPE, "application/json, text/plain, */*")
        .header(COOKIE, &format!("SESSDATA={}", cookies.sessdata))
        .send()?;

    let res: serde_json::Value = response.json()?;

    if res["code"].as_i64() != Some(0) {
        return Err(BiliLiveError::ApiError(
            format!(
                "API返回错误: {}",
                res["message"].as_str().unwrap_or("未知错误")
            )
            .into(),
        ));
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
