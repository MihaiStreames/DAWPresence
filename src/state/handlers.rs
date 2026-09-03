use iced::Task;
use iced::window;
use tracing::debug;
use tracing::warn;

use super::AppState;
use super::Message;
use super::Page;
use super::save_or_warn;
use crate::daw::DawScanner;
use crate::settings::AppSettings;
use crate::ui::tray::TrayUpdate;
use crate::ui::tray::send_tray_update;
use crate::win32::autostart;

const INTERVAL_PARSE_ERROR: &str = "Interval must be a number";

pub(super) fn close_requested(settings: &AppSettings, window_id: window::Id) -> Task<Message> {
    if settings.close_to_tray {
        window::set_mode(window_id, window::Mode::Hidden)
    } else {
        window::close(window_id)
    }
}

pub(super) fn window_opened(state: &mut AppState, window_id: window::Id) -> Task<Message> {
    state.window_id = Some(window_id);

    // TODO: test new minimized logic first before deciding to remove this
    if state.start_minimized {
        return window::set_mode(window_id, window::Mode::Hidden);
    }

    Task::none()
}

pub(super) fn tray_show(window_id: Option<window::Id>) -> Task<Message> {
    let Some(id) = window_id else {
        return Task::none();
    };

    Task::batch(vec![
        window::set_mode(id, window::Mode::Windowed),
        window::gain_focus(id),
    ])
}

pub(super) fn tray_quit(window_id: Option<window::Id>) -> Task<Message> {
    let Some(id) = window_id else {
        return Task::none();
    };

    window::close(id)
}

pub(super) fn navigate_to(state: &mut AppState, page: Page) -> Task<Message> {
    state.active_page = page;
    debug!("Navigated to {page:?}");

    if page == Page::Settings {
        state.update_interval_input = state.settings.update_interval.to_string();
        state.update_interval_error = None;
    }

    Task::none()
}

pub(super) fn toggle_auto_start(state: &mut AppState, value: bool) -> Task<Message> {
    autostart::set_enabled(value);
    // re-read registry to confirm write succeeded
    state.auto_start_enabled = autostart::is_enabled();
    debug!("Auto-start toggled: {}", state.auto_start_enabled);
    Task::none()
}

pub(super) fn toggle_close_to_tray(state: &mut AppState, value: bool) -> Task<Message> {
    state.settings.close_to_tray = value;
    debug!("Close to tray toggled: {value}");
    save_or_warn(&state.settings);
    Task::none()
}

pub(super) fn toggle_hide_project_name(state: &mut AppState, value: bool) -> Task<Message> {
    state.settings.hide_project_name = value;
    debug!("Hide project name toggled: {value}");
    save_or_warn(&state.settings);
    send_tray_update(TrayUpdate::HideProjectName(value));
    Task::none()
}

pub(super) fn toggle_hide_system_usage(state: &mut AppState, value: bool) -> Task<Message> {
    state.settings.hide_system_usage = value;
    debug!("Hide system usage toggled: {value}");
    save_or_warn(&state.settings);
    send_tray_update(TrayUpdate::HideSystemUsage(value));
    Task::none()
}

pub(super) fn update_interval_input(state: &mut AppState, value: &str) -> Task<Message> {
    value.clone_into(&mut state.update_interval_input);
    state.interval_applied = false;

    if value.trim().is_empty() {
        state.update_interval_error = Some(INTERVAL_PARSE_ERROR.to_owned());
    } else if let Ok(interval) = value.parse::<u64>() {
        match AppSettings::validate_update_interval(interval) {
            Ok(()) => state.update_interval_error = None,

            Err(error) => state.update_interval_error = Some(error.to_string()),
        }
    } else {
        state.update_interval_error = Some(INTERVAL_PARSE_ERROR.to_owned());
    }

    Task::none()
}

pub(super) fn apply_interval(state: &mut AppState) -> Task<Message> {
    let Ok(interval) = state.update_interval_input.parse::<u64>() else {
        state.update_interval_error = Some(INTERVAL_PARSE_ERROR.to_owned());
        return Task::none();
    };

    if let Err(error) = state.settings.set_update_interval(interval) {
        state.update_interval_error = Some(error.to_string());
        return Task::none();
    }

    state.update_interval_error = None;
    state.interval_applied = true;
    debug!("Update interval applied: {interval}ms");
    save_or_warn(&state.settings);
    Task::none()
}

pub(super) fn tick(state: &mut AppState) -> Task<Message> {
    state.interval_applied = false;

    let status = state.daw_scanner.as_mut().and_then(DawScanner::poll);
    state.daw_status = status;

    if let Err(error) = state
        .discord
        .update_from_daw_status(state.daw_status.as_ref(), &state.settings)
    {
        warn!("Couldn't update Discord presence: {error}");
    }

    let connected = state.discord.is_connected();
    if connected != state.discord_connected {
        debug!("Discord connection state: {connected}");
        state.discord_connected = connected;
        send_tray_update(TrayUpdate::DiscordConnected(connected));
    }

    Task::none()
}
