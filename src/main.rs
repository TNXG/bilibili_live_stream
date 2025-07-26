mod utils;
mod error;
mod logger;

use error::{BiliLiveError, Result};
use crate::logger::init_logger;

fn main() {
    // 初始化日志系统
    init_logger();
    
    if let Err(e) = run() {
        user_error!("程序执行失败: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let check_status = utils::check_status()?;

    if !check_status {
        user_info!("需要登录，开始登录流程...");
        utils::start_login()?;
        user_success!("登录成功！");
    } else {
        user_success!("登录状态正常");
    }

    user_prompt!("是否使用上次直播的分区？(直接回车或输入y使用默认，输入n选择新分区)");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| BiliLiveError::InputError(format!("读取用户输入失败: {}", e)))?;

    let area_id = if input.trim().is_empty() || input.trim().to_lowercase() == "y" {
        let (id, name) = utils::get_recent_live()?;
        user_success!("使用上次的分区: {} - {}", name, id);
        id.parse().map_err(|e| BiliLiveError::ParseError(format!("分区ID转换失败: {}", e)))?
    } else {
        user_info!("选择合适的直播分区！");
        utils::get_area_choice()?
    };

    // 开始直播
    user_info!("开始直播！");
    let live_id = utils::start_live(&area_id.to_string())?;

    user_info!("请在本程序中按 Ctrl+C 关闭直播！否则直播将不会关闭！");

    // 监听程序退出信号
    let _ = ctrlc::set_handler(move || {
        user_info!("监听到 Ctrl+C，准备关闭直播！");
        if let Err(e) = utils::stop_live(live_id) {
            user_error!("停止直播失败: {}", e);
        } else {
            user_success!("直播已关闭！");
        }
        std::process::exit(0);
    });

    // 使程序保持运行状态
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
