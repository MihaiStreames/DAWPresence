//! DAW detection domain: process scanning, status tracking, config loading.

mod config;
mod regex_cache;
mod scanner;
mod status;

#[cfg(windows)]
#[allow(unsafe_code)]
mod win32;

pub(crate) use config::ensure_daw_config;
pub(crate) use config::load_configs;
pub(crate) use scanner::DawScanner;
pub(crate) use status::DawStatus;
pub(crate) use status::UNKNOWN_PROJECT;
pub(crate) use status::UNKNOWN_VERSION;
pub(crate) use status::UNTITLED_PROJECT;
