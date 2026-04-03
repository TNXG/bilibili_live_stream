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
