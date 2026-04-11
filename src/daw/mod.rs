//! DAW detection domain: process scanning, status tracking, config loading.

pub(crate) mod config;
mod regex_cache;
pub(crate) mod scanner;
pub(crate) mod status;

#[cfg(windows)]
#[allow(unsafe_code)]
mod win32;

pub(crate) use config::ensure_daw_config;
pub(crate) use config::load_configs;
pub(crate) use scanner::DawScanner;
pub(crate) use status::DawStatus;
