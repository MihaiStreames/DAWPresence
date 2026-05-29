#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

#[cfg(not(windows))]
compile_error!("DAWPresence is Windows-only");

mod daw;
mod discord;
mod error;
mod settings;
mod state;
mod ui;

#[cfg(windows)]
#[allow(unsafe_code)]
mod win32;

use iced::Size;
use iced::window;
use tracing::info;
#[cfg(not(debug_assertions))]
use tracing_subscriber as _;
#[cfg(debug_assertions)]
use tracing_subscriber::EnvFilter;

use crate::ui::tray::load_window_icon;

#[cfg(windows)]
fn main() -> iced::Result {
    init_logging();

    info!("DAWPresence v{} starting up", env!("CARGO_PKG_VERSION"));

    if !win32::single_instance::acquire() {
        return Ok(());
    }

    let window_icon = load_window_icon().ok();

    iced::application(state::boot, state::update, view)
        .title("DAWPresence")
        .subscription(state::subscription)
        .window(window::Settings {
            resizable: false,
            icon: window_icon,
            size: Size::new(784.0, 340.0),
            exit_on_close_request: false,
            ..window::Settings::default()
        })
        .run()
}

#[cfg(debug_assertions)]
fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("DAWPRESENCE_LOG")
                .unwrap_or_else(|_| EnvFilter::new("DAWPresence=debug,warn")),
        )
        .init();
}

#[cfg(not(debug_assertions))]
fn init_logging() {}

fn view(state: &state::AppState) -> iced::Element<'_, state::Message> {
    ui::view(state)
}
