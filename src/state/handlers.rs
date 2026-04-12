//! Message handlers for application state updates.

use iced::Task;
use iced::window;
use tracing::warn;

use super::AppState;
use super::Message;
use super::save_or_warn;
use crate::daw::DawScanner;
use crate::settings::AppSettings;
use crate::ui::tray::TrayUpdate;

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

pub(super) fn toggle_close_to_tray(state: &mut AppState, value: bool) -> Task<Message> {
    state.settings.close_to_tray = value;
    save_or_warn(&state.settings);
    Task::none()
}

pub(super) fn toggle_hide_project_name(state: &mut AppState, value: bool) -> Task<Message> {
    state.settings.hide_project_name = value;
    save_or_warn(&state.settings);
    crate::ui::tray::send_tray_update(TrayUpdate::HideProjectName(value));
    Task::none()
}

pub(super) fn toggle_hide_system_usage(state: &mut AppState, value: bool) -> Task<Message> {
    state.settings.hide_system_usage = value;
    save_or_warn(&state.settings);
    crate::ui::tray::send_tray_update(TrayUpdate::HideSystemUsage(value));
    Task::none()
}

pub(super) fn update_interval_input(state: &mut AppState, value: &str) -> Task<Message> {
    state.update_interval_input = value.to_string();

    if value.trim().is_empty() {
        state.update_interval_error = Some(INTERVAL_PARSE_ERROR.to_string());
    } else if let Ok(interval) = value.parse::<u64>() {
        match AppSettings::validate_update_interval(interval) {
            Ok(()) => state.update_interval_error = None,

            Err(error) => state.update_interval_error = Some(error.to_string()),
        }
    } else {
        state.update_interval_error = Some(INTERVAL_PARSE_ERROR.to_string());
    }

    Task::none()
}

pub(super) fn open_interval_modal(state: &mut AppState) -> Task<Message> {
    state.update_interval_input = state.settings.update_interval.to_string();
    state.update_interval_error = None;
    state.modal_dismiss_warning = false;
    state.show_interval_modal = true;
    Task::none()
}

pub(super) fn close_interval_modal(state: &mut AppState) -> Task<Message> {
    state.show_interval_modal = false;
    state.modal_dismiss_warning = false;
    Task::none()
}

pub(super) fn overlay_clicked(state: &mut AppState) -> Task<Message> {
    if !state.show_interval_modal {
        return Task::none();
    }

    let original = state.settings.update_interval.to_string();
    if state.update_interval_input == original {
        state.show_interval_modal = false;
        state.modal_dismiss_warning = false;
    } else {
        state.modal_dismiss_warning = true;
    }

    Task::none()
}

pub(super) fn apply_interval(state: &mut AppState) -> Task<Message> {
    let Ok(interval) = state.update_interval_input.parse::<u64>() else {
        state.update_interval_error = Some(INTERVAL_PARSE_ERROR.to_string());
        return Task::none();
    };

    if let Err(error) = state.settings.set_update_interval(interval) {
        state.update_interval_error = Some(error.to_string());
        return Task::none();
    }

    state.update_interval_error = None;
    if let Err(error) = state.settings.save() {
        warn!("Couldn't save settings: {error}");
    }

    state.show_interval_modal = false;
    Task::none()
}

pub(super) fn tick(state: &mut AppState) -> Task<Message> {
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
        state.discord_connected = connected;
        crate::ui::tray::send_tray_update(TrayUpdate::DiscordConnected(connected));
    }

    Task::none()
}
