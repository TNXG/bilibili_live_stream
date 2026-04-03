use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

pub fn init_logger() {
    let mut builder = Builder::new();

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

    builder.filter(None, LevelFilter::Info);

    builder.target(Target::Stdout);

    builder.init();
}
