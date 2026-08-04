use crate::api::passport::LoginClient;
use crate::auth::cookies::save_cookies;
use crate::error::{BiliLiveError, Result};
use crate::utils::qrcode::{generate_and_save_qrcode, print_qrcode_in_terminal};
use crate::{user_info, user_success, user_warning};

pub struct QRStatus {
    pub waiting: i32,
    pub scanned: i32,
    pub success: i32,
}

pub const QR_STATUS: QRStatus = QRStatus {
    waiting: 86101,
    scanned: 86090,
    success: 0,
};

pub fn start_login() -> Result<()> {
    user_info!("开始B站二维码登录流程...");

    let login_client = LoginClient::new()?;
    let mut qr_data = login_client.generate_qr_code()?;
    loop {
        user_info!("请使用B站手机客户端如下链接：{}", qr_data.url);
        user_info!("或使用B站手机客户端扫描如下二维码");

        print_qrcode_in_terminal(&qr_data.url)?;
        generate_and_save_qrcode(&qr_data.url, "qrcode.png")?;
        user_success!("二维码已保存到 qrcode.png");

        user_info!("等待用户处理...");

        loop {
            let poll_data = login_client.poll_qr_status(&qr_data.qrcode_key)?;

            match poll_data.code {
                code if code == QR_STATUS.waiting => {}
                code if code == QR_STATUS.scanned => {
                    user_info!("已处理，请在手机上确认登录");
                }
                code if code == QR_STATUS.success => {
                    user_success!("登录成功！");
                    let (sessdata, csrf) = login_client.finish_login(&poll_data.url)?;
                    save_cookies(&sessdata, &csrf)?;
                    std::fs::remove_file("qrcode.png")?;
                    return Ok(());
                }
                86038 => {
                    user_warning!("二维码已失效，正在重新获取...");
                    break;
                }
                _ => {
                    return Err(BiliLiveError::Api(format!(
                        "登录失败：{} (code={})",
                        poll_data.message, poll_data.code
                    )));
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        // 重新获取二维码
        qr_data = login_client.generate_qr_code()?;
    }
}
