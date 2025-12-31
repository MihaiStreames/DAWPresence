use std::sync::OnceLock;
use std::time::Instant;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn label(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }
}

static START_TIME: OnceLock<Instant> = OnceLock::new();
static LOG_LEVEL: OnceLock<LogLevel> = OnceLock::new();

fn elapsed_ms() -> u128 {
    START_TIME.get_or_init(Instant::now).elapsed().as_millis()
}

fn parse_level(value: &str) -> Option<LogLevel> {
    match value.trim().to_lowercase().as_str() {
        "error" => Some(LogLevel::Error),
        "warn" | "warning" => Some(LogLevel::Warn),
        "info" => Some(LogLevel::Info),
        "debug" => Some(LogLevel::Debug),
        "trace" => Some(LogLevel::Trace),
        _ => None,
    }
}

fn default_level() -> LogLevel {
    if cfg!(debug_assertions) {
        LogLevel::Debug
    } else {
        LogLevel::Info
    }
}

pub fn current_level() -> LogLevel {
    *LOG_LEVEL.get_or_init(|| {
        std::env::var("DAWPRESENCE_LOG")
            .ok()
            .and_then(|v| parse_level(&v))
            .unwrap_or_else(default_level)
    })
}

pub fn log(level: LogLevel, message: impl AsRef<str>) {
    if level > current_level() {
        return;
    }

    let ms = elapsed_ms();
    println!("[{ms:>7}ms][{}] {}", level.label(), message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    log(LogLevel::Error, message);
}

pub fn warn(message: impl AsRef<str>) {
    log(LogLevel::Warn, message);
}

pub fn info(message: impl AsRef<str>) {
    log(LogLevel::Info, message);
}

pub fn debug(message: impl AsRef<str>) {
    log(LogLevel::Debug, message);
}

pub fn trace(message: impl AsRef<str>) {
    log(LogLevel::Trace, message);
}

