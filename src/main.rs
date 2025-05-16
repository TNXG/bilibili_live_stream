mod utils;

fn main() {
    let check_status = utils::check_status();

    if !check_status {
        utils::start_login().expect("登录失败");
    }

    println!("是否使用上次直播的分区？(直接回车或输入y使用默认，输入n选择新分区)");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("读取输入失败");

    let area_id = if input.trim().is_empty() || input.trim().to_lowercase() == "y" {
        let (id, name) = utils::get_recent_live().expect("获取上次分区失败");
        println!("使用上次的分区: {} - {}", name, id);
        id.parse().expect("分区ID转换失败")
    } else {
        println!("选择合适的直播分区！");
        utils::get_area_choice().expect("分区选择失败")
    };

    // 开始直播
    println!("开始直播！");
    let live_id = utils::start_live(&area_id.to_string()).expect("直播失败");

    // 监听程序退出信号
    let _ = ctrlc::set_handler(move || {
        println!("退出直播！");
        utils::stop_live(live_id).expect("停止直播失败");
        std::process::exit(0);
    });

    // 使程序保持运行状态
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
