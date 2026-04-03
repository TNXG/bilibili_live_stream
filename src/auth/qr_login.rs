use crate::api::passport::{generate_qr_code, poll_qr_status};
use crate::auth::cookies::save_cookies;
use crate::error::Result;
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

    let qr_data = generate_qr_code()?;
    user_info!("请使用B站手机客户端如下链接：{}", qr_data.url);

    user_info!("或使用B站手机客户端扫描如下二维码");

    print_qrcode_in_terminal(&qr_data.url)?;

    generate_and_save_qrcode(&qr_data.url, "qrcode.png")?;
    user_success!("二维码已保存到 qrcode.png");

    user_info!("等待用户处理...");

    loop {
        let poll_data = poll_qr_status(&qr_data.qrcode_key)?;

        match poll_data.code {
            code if code == QR_STATUS.waiting => {}
            code if code == QR_STATUS.scanned => {
                user_info!("已处理，请在手机上确认登录");
            }
            code if code == QR_STATUS.success => {
                user_success!("登录成功！");
                save_cookies(&poll_data.url)?;
                std::fs::remove_file("qrcode.png")?;
                break;
            }
            _ => {
                user_warning!("未知状态：{}", poll_data.message);
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    Ok(())
}
