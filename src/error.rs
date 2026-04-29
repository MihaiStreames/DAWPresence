use thiserror::Error;

/// Configuration loading and validation errors.
#[derive(Error, Debug)]
pub(crate) enum ConfigError {
    #[error("Couldn't read daws.json: {0}")]
    ReadFailed(#[from] std::io::Error),

    #[error("Couldn't parse daws.json: {0}")]
    ParseFailed(#[from] serde_json::Error),

    #[error("Couldn't resolve config directory")]
    NoConfigDir,

    #[error("Couldn't initialize config: {0}")]
    InitFailed(String),

    #[error("Interval must be between {min}ms and {max}ms")]
    InvalidInterval { min: u64, max: u64 },

    #[error("Couldn't save settings: {0}")]
    SaveFailed(String),
}

/// Discord IPC connection errors.
#[derive(Error, Debug)]
pub(crate) enum DiscordError {
    #[error("Couldn't connect to Discord: {0}")]
    Connect(String),

    #[error("set_activity failed: {0}")]
    Activity(String),

    #[error("set_activity failed: {activity}; reconnect also failed: {reconnect}")]
    Reconnect { activity: String, reconnect: String },
}

/// Tray icon errors.
#[derive(Error, Debug)]
pub(crate) enum TrayError {
    #[error("Couldn't create tray icon: {0}")]
    CreateFailed(String),

    #[error("Couldn't load icon: {0}")]
    IconFailed(String),
}
