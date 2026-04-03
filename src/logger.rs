use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

pub fn init_logger() {
    let mut builder = Builder::new();

    // 设置日志格式
    builder.format(|buf, record| {
        let level_emoji = match record.level() {
            log::Level::Error => "❌",
            log::Level::Warn => "⚠️",
            log::Level::Info => "ℹ️",
            log::Level::Debug => "🔍",
            log::Level::Trace => "🔬",
        };

        writeln!(
            buf,
            "{} {} [{}] {}",
            level_emoji,
            chrono::Local::now().format("%H:%M:%S"),
            record.target(),
            record.args()
        )
    });

    // 设置默认日志级别
    builder.filter(None, LevelFilter::Info);

    // 设置目标为stdout
    builder.target(Target::Stdout);

    // 初始化日志系统
    builder.init();
}

// 用户友好的日志宏
#[macro_export]
macro_rules! user_info {
    ($($arg:tt)*) => {
        log::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! user_success {
    ($($arg:tt)*) => {
        log::info!("✅ {}", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! user_warning {
    ($($arg:tt)*) => {
        log::warn!("⚠️ {}", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! user_error {
    ($($arg:tt)*) => {
        log::error!("❌ {}", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! user_prompt {
    ($($arg:tt)*) => {
        {
            use std::io::Write;
            print!("📺 {}", format!($($arg)*));
            std::io::stdout().flush().unwrap();
        }
    };
}

#[macro_export]
macro_rules! user_input_prompt {
    ($($arg:tt)*) => {
        {
            use std::io::Write;
            print!("🎯 {}", format!($($arg)*));
            std::io::stdout().flush().unwrap();
        }
    };
}
