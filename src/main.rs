//! DAWPresence entry point, logging setup, and Iced wiring.

#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod daw;
mod discord;
mod error;
mod settings;
mod state;
mod ui;
mod version;

use iced::Size;
use iced::window;
use tracing::info;
#[cfg(debug_assertions)]
use tracing_subscriber::EnvFilter;

use crate::ui::tray::load_window_icon;

#[cfg(windows)]
fn main() -> iced::Result {
    init_logging();

    info!("DAWPresence v{} starting up", version::APP_VERSION);

    let window_icon = load_window_icon().ok();

    iced::application(state::boot, state::update, view)
        .title("DAWPresence")
        .subscription(state::subscription)
        .window(window::Settings {
            resizable: false,
            icon: window_icon,
            size: Size::new(784.0, 300.0),
            exit_on_close_request: false,
            ..window::Settings::default()
        })
        .run()
}

#[cfg(not(windows))]
fn main() -> ! {
    eprintln!("DAWPresence is Windows-only (for now)");
    std::process::exit(1);
}

#[cfg(debug_assertions)]
fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("DAWPRESENCE_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

#[cfg(not(debug_assertions))]
fn init_logging() {}

/// Render the main window content.
fn view(state: &state::AppState) -> iced::Element<'_, state::Message> {
    ui::view(state)
}
