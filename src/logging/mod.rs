use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use log::{Level, LevelFilter, Log, Metadata, Record};

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: Level,
    pub time: String,
    pub message: String,
}

#[derive(Clone)]
pub struct LogBuffer {
    inner: Arc<Mutex<VecDeque<LogEntry>>>,
}

impl LogBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(800))),
        }
    }

    pub fn push(&self, level: Level, message: String) {
        let mut entries = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if entries.len() == 800 {
            entries.pop_front();
        }
        entries.push_back(LogEntry {
            level,
            time: chrono::Local::now().format("%H:%M:%S").to_string(),
            message,
        });
    }

    pub fn entries(&self) -> MutexGuard<'_, VecDeque<LogEntry>> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new()
    }
}

struct BufferLogger(LogBuffer);

impl Log for BufferLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }
    fn log(&self, record: &Record<'_>) {
        // HTTP debug traces may contain cookies or signed URLs. Only application logs enter the UI.
        if self.enabled(record.metadata()) && record.target().starts_with("bili_live") {
            self.0.push(record.level(), record.args().to_string());
        }
    }
    fn flush(&self) {}
}

pub fn init(buffer: LogBuffer) -> Result<(), log::SetLoggerError> {
    let level = std::env::var("BILI_LOG")
        .ok()
        .and_then(|value| value.parse::<LevelFilter>().ok())
        .unwrap_or(LevelFilter::Info);
    log::set_boxed_logger(Box::new(BufferLogger(buffer)))?;
    log::set_max_level(level);
    Ok(())
}
