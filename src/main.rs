// prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod daw;
mod discord;
mod settings;

use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("DAWPRESENCE_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("DAWPresence v{} starting up", env!("CARGO_PKG_VERSION"));

    // TODO: setup tray icon
    // TODO: setup iced window
    // TODO: start update loop
}
