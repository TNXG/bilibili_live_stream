mod api;
mod auth;
mod error;
mod live;
mod logger;
mod ui;
mod utils;

use crate::logger::init_logger;
use clap::Parser;
use error::{BiliLiveError, Result};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, help = "显示完整的推流码，不进行打码处理")]
    show_full_code: bool,
}

fn main() {
    let args = Args::parse();

    init_logger();

    if let Err(e) = run(args) {
        user_error!("程序执行失败: {}", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let check_status = auth::check_status()?;

    if !check_status {
        user_info!("需要登录，开始登录流程...");
        auth::start_login()?;
        user_success!("登录成功！");
    } else {
        user_success!("登录状态正常");
    }

    user_prompt!("是否使用上次直播的分区？(直接回车或输入y使用默认，输入n选择新分区)");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| BiliLiveError::Input(format!("读取用户输入失败: {}", e)))?;

    let area_id = if input.trim().is_empty() || input.trim().to_lowercase() == "y" {
        let (id, name) = api::live::get_recent_live()?;
        user_success!("使用上次的分区: {} - {}", name, id);
        id.parse()
            .map_err(|e| BiliLiveError::Parse(format!("分区ID转换失败: {}", e)))?
    } else {
        user_info!("选择合适的直播分区！");
        ui::get_area_choice()?
    };

    user_info!("开始直播！");
    let live_id = live::start_live(&area_id.to_string(), args.show_full_code)?;

    user_info!("请在本程序中按 Ctrl+C 关闭直播！否则直播将不会关闭！");

    let _ = ctrlc::set_handler(move || {
        user_info!("监听到 Ctrl+C，准备关闭直播！");
        if let Err(e) = live::stop_live(live_id) {
            user_error!("停止直播失败: {}", e);
        } else {
            user_success!("直播已关闭！");
        }
        std::process::exit(0);
    });

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
