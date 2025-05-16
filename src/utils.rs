use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cookies {
    pub room_id: i32,
    pub sessdata: String,
    pub csrf_token: String,
}

#[derive(Debug, Deserialize)]
struct QRKeyResponseData {
    url: String,
    qrcode_key: String,
}

#[derive(Debug, Deserialize)]
struct QRKeyResponse {
    data: QRKeyResponseData,
}

#[derive(Debug, Deserialize)]
struct QrPollResponseData {
    url: String,
    code: i32,
    message: String,
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

pub struct QRStatus {
    pub waiting: i32,
    pub scanned: i32,
    pub success: i32,
}

pub const QR_STATUS: QRStatus = QRStatus {
    waiting: 86101, // 等待扫码
    scanned: 86090, // 已扫码，等待确认
    success: 0,     // 登录成功
};

fn generate_qr_code() -> Result<QRKeyResponseData, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .send()?
        .json::<QRKeyResponse>()?;
    Ok(response.data)
}

fn poll_qr_status(
    qrcode_key: &str,
) -> Result<QrPollResponseData, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/poll")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .query(&[("qrcode_key", qrcode_key)])
        .send()?
        .json::<QRPollResponse>()?;
    Ok(response.data)
}

pub fn get_query_string(name: &str, url: &str) -> String {
    let pairs: Vec<&str> = url.split('?').nth(1).unwrap_or("").split('&').collect();

    for pair in pairs {
        let mut parts = pair.split('=');
        if let Some(key) = parts.next() {
            if key == name {
                return parts.next().unwrap_or("").to_string();
            }
        }
    }
    String::new()
}

pub fn get_roomid(sessdata: &str) -> i32 {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .send()
        .expect("发送请求错误");
    
    // 自动解析 JSON 到 NavResponse
    let nav_response: NavResponse = response.json().expect("解析 JSON 错误");
    let user_code = nav_response.data.mid.to_string();  // 转换为字符串（如果需要）

    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.live.bilibili.com/room/v1/Room/getRoomInfoOld")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .query(&[("mid", user_code)])
        .send()
        .expect("发送请求错误");
    
    // 自动解析 JSON 到 RoomInfoResponse
    let room_info: RoomInfoResponse = response.json().expect("解析 JSON 错误");
    room_info.data.roomid as i32
}
pub fn save_cookies(set_cookies_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bili_sessdata = get_query_string("SESSDATA", set_cookies_url);
    let csrf = get_query_string("bili_jct", set_cookies_url);
    let cookies = Cookies {
        room_id: get_roomid(&bili_sessdata),
        sessdata: bili_sessdata,
        csrf_token: csrf,
    };

    let cookies_json = serde_json::to_string_pretty(&cookies)?;
    fs::write("cookies.json", cookies_json)?;
    println!("Cookies保存成功");
    Ok(())
}

pub fn read_cookies() -> Result<Cookies, Box<dyn std::error::Error>> {
    let cookies_str = std::fs::read_to_string("./cookies.json").expect("读取cookies.json失败");
    let cookies: Cookies = serde_json::from_str(&cookies_str).expect("解析cookies.json失败");
    Ok(cookies)
}

pub fn check_status() -> bool {
    println!("检查登录状态...");
    // 先检查一下文件是否存在
    if !std::path::Path::new("cookies.json").exists() {
        println!("cookies.json文件不存在");
        return false;
    }
    // 检查一下文件内容是否为空
    if std::fs::read_to_string("cookies.json").unwrap().is_empty() {
        println!("cookies.json文件为空");
        return false;
    }
    // 读取cookies.json文件
    let sessdata = read_cookies().expect("读取cookies.json错误").sessdata;
    // 发送请求
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .send()
        .expect("发送请求错误");
    // 解析响应
    let response_json: serde_json::Value = serde_json::from_str(&response.text().unwrap()).unwrap();
    let code = response_json["code"].as_i64().unwrap();
    if code == 0 {
        return true;
    } else {
        println!("登录状态异常");
        return false;
    }
}

pub fn get_area_choice() -> Result<u32, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.live.bilibili.com/room/v1/Area/getList")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .send()?;
    
    let area_list: serde_json::Value = response.json()?;
    
    loop {
        // 显示一级分区
        println!("\n一级分区列表:");
        if let Some(data) = area_list["data"].as_array() {
            for (i, area) in data.iter().enumerate() {
                println!("{}. {}", i+1, area["name"]);
            }
        }
        
        println!("\n请输入一级分区编号:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let first_choice: usize = input.trim().parse()?;
        
        if first_choice == 0 {
            println!("你是抱着多大的觉悟在一级菜单按下“0”的？");
            continue;
        }
        
        if let Some(data) = area_list["data"].as_array() {
            if first_choice > 0 && first_choice <= data.len() {
                let selected_first_area = &data[first_choice-1];
                
                // 显示二级分区
                if let Some(second_list) = selected_first_area["list"].as_array() {
                    loop {
                        println!("\n二级分区列表 ({}):", selected_first_area["name"]);
                        for (i, area) in second_list.iter().enumerate() {
                            println!("{}. {} - {}", i+1, area["name"], area["id"]);
                        }
                        
                        println!("\n请输入二级分区编号(输入0返回):");
                        let mut second_input = String::new();
                        std::io::stdin().read_line(&mut second_input)?;
                        let second_choice: usize = second_input.trim().parse()?;
                        
                        if second_choice == 0 {
                            break;
                        }
                        
                        if second_choice > 0 && second_choice <= second_list.len() {
                            let selected_area = &second_list[second_choice-1];
                            println!("\n已选择分区: {} (ID: {})", selected_area["name"], selected_area["id"]);
                            let id_str = selected_area["id"].as_str().unwrap_or("");
                            let numeric_id: String = id_str.chars().filter(|c| c.is_numeric()).collect();
                            return Ok(numeric_id.parse::<u32>()?);
                        }
                        
                        println!("无效的选择，请重新输入");
                    }
                }
            }
        }
        
        println!("无效的选择，请重新输入");
    }
}

pub fn start_login() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始B站二维码登录流程...");

    let qr_data = generate_qr_code()?;
    println!("请使用B站手机客户端如下链接：{}", qr_data.url);

    println!("或使用B站手机客户端扫描如下二维码");

    print_qrcode_in_terminal(&qr_data.url)?;

    // 生成二维码图片并保存到本地
    generate_and_save_qrcode(&qr_data.url, "qrcode.png")?;
    println!("二维码已保存到 qrcode.png");

    println!("等待用户处理...");

    // 轮询扫码状态
    loop {
        let poll_data = poll_qr_status(&qr_data.qrcode_key).expect("轮询登录状态失败");

        match poll_data.code {
            code if code == QR_STATUS.waiting => {
                // 可以添加等待提示
            }
            code if code == QR_STATUS.scanned => {
                println!("已处理，请在手机上确认登录");
            }
            code if code == QR_STATUS.success => {
                println!("登录成功！");
                save_cookies(&poll_data.url)?;
                std::fs::remove_file("qrcode.png")?;
                break;
            }
            _ => {
                println!("未知状态：{}", poll_data.message);
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    Ok(())
}

/// 生成二维码图片并保存到文件
fn generate_and_save_qrcode(url: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    use qrcode::QrCode;
    use image::Luma;
    use std::path::Path;

    // 生成二维码
    let code = QrCode::new(url.as_bytes())?;
    
    // 转换为图像
    let image = code.render::<Luma<u8>>()
        .quiet_zone(false)  // 禁用静区（可选）
        .min_dimensions(200, 200)  // 最小尺寸
        .build();
    
    // 保存为PNG文件
    let path = Path::new(filename);
    image.save(path)?;
    
    Ok(())
}

fn print_qrcode_in_terminal(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use qrcode::QrCode;
    let code = QrCode::new(url.as_bytes())?;
    
    // 转换为ASCII字符串
    let string = code.render()
        .light_color(' ')  // 浅色部分用空格
        .dark_color('█')  // 深色部分用方块
        .quiet_zone(false)
        .build();
    
    println!("{}", string);
    Ok(())
}

// 获取用户最近直播过的分区信息
pub fn get_recent_live() -> Result<(String, String), Box<dyn std::error::Error>> {
    let room_id = read_cookies().expect("读取cookies.json错误").room_id;
    let client = reqwest::blocking::Client::new();
    let res = client.get("https://api.live.bilibili.com/room/v1/Area/getMyChooseArea")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .query(&[("roomid", room_id.to_string())])
        .send()?;
    let json: serde_json::Value = res.json()?;
    let data = &json["data"][0];
    let id = data["id"].as_str().unwrap().to_string();
    let name = data["name"].as_str().unwrap().to_string();
    Ok((id, name))
}

// 开始直播，获取推流码和推流地址
pub fn start_live(area_id: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let cookies = read_cookies().expect("读取cookies.json错误");
    let client = reqwest::blocking::Client::new();
    let res = client.post("https://api.live.bilibili.com/room/v1/Room/startLive")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("room_id", cookies.room_id.to_string()),
            ("area_v2", area_id.to_string()),
            ("csrf", cookies.csrf_token.clone()),
            ("platform", "pc_link".to_string()),
        ])
        .header("Cookie", format!("SESSDATA={}", cookies.sessdata))
        .send()?
        .json::<serde_json::Value>()?;

    if res["code"].as_i64() != Some(0) {
        return Err(format!("API返回错误: {}", res["message"].as_str().unwrap_or("未知错误")))?;
    }

    let rtmp_addr = res["data"]["rtmp"]["addr"].as_str().ok_or("缺少rtmp地址")?;
    let rtmp_code = res["data"]["rtmp"]["code"].as_str().ok_or("缺少rtmp code")?;
    let live_key = res["data"]["live_key"].as_str().ok_or("缺少live_key")?;
    
    println!("RTMP地址: {}", rtmp_addr);
    println!("直播码: {}", rtmp_code);

    Ok(live_key.parse::<u64>()?)
}

pub fn stop_live(live_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    let cookies = read_cookies().expect("读取cookies.json错误");
    let client = reqwest::blocking::Client::new();
    let res = client.post("https://api.live.bilibili.com/room/v1/Room/stopLive")
        .header("User-Agent", DEFAULT_USER_AGENT)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("room_id", cookies.room_id.to_string()),
            ("csrf", cookies.csrf_token.clone()),
        ])
        .header("Cookie", format!("SESSDATA={}", cookies.sessdata))
        .send()?
        .json::<serde_json::Value>()?;

    if res["code"].as_i64() != Some(0) {
        return Err(format!("API返回错误: {}", res["message"].as_str().unwrap_or("未知错误")))?;
    }
    
    println!("成功关闭直播");

    get_live_info(live_id)?;

    Ok(())
}

fn get_live_info(live_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    let cookies = read_cookies().expect("读取cookies.json错误");
    let client = reqwest::blocking::Client::new();
    let res = client.get("https://api.live.bilibili.com/xlive/app-blink/v1/live/StopLiveData")
       .header("User-Agent", DEFAULT_USER_AGENT)
       .header("Content-Type", "application/json, text/plain, */*")
       .header("Cookie", format!("SESSDATA={}", cookies.sessdata))
       .query(&[("live_key", live_id.to_string())])
       .send()?
      .json::<serde_json::Value>()?;
    
    if res["code"].as_i64() != Some(0) {
        return Err(format!("API返回错误: {}", res["message"].as_str().unwrap_or("未知错误")))?;
    }
    
    let data = &res["data"];
    println!("直播统计信息:");
    println!("新增粉丝 : {}", data["AddFans"].as_i64().unwrap_or(0));
    println!("弹幕数 : {}", data["DanmuNum"].as_i64().unwrap_or(0));
    println!("金仓鼠流水 : {}", data["HamsterRmb"].as_i64().unwrap_or(0));
    println!("直播时长 : {}", data["LiveTime"].as_i64().unwrap_or(0));
    println!("最大在线 : {}", data["MaxOnline"].as_i64().unwrap_or(0));
    println!("新增粉丝勋章 : {}", data["NewFansClub"].as_i64().unwrap_or(0));
    println!("累计观看 : {}", data["WatchedCount"].as_i64().unwrap_or(0));
    
    Ok(())
}