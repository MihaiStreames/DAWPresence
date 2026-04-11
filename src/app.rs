//! Application state machine.

use std::time::Duration;

use iced::Subscription;
use iced::Task;
use iced::event;
use iced::time;
use iced::window;
use tracing::warn;

use crate::daw::DawScanner;
use crate::daw::DawStatus;
use crate::daw::ensure_daw_config;
use crate::discord::DiscordManager;
use crate::settings::AppSettings;
use crate::tray::TrayUpdate;
use crate::tray::tray_subscription;

/// Events flowing through the Iced update loop.
#[derive(Debug, Clone)]
pub(crate) enum Message {
    CloseRequested(window::Id),
    WindowOpened(window::Id),
    TrayShow,
    TrayQuit,
    ToggleCloseToTray(bool),
    ToggleHideProjectName(bool),
    ToggleHideSystemUsage(bool),
    UpdateIntervalInput(String),
    OpenIntervalModal,
    CloseIntervalModal,
    ApplyInterval,
    Tick,
}

/// Root application state for the Iced MVU loop.
pub(crate) struct AppState {
    pub(crate) settings: AppSettings,
    pub(crate) update_interval_input: String,
    pub(crate) update_interval_error: Option<String>,
    pub(crate) show_interval_modal: bool,
    pub(crate) daw_status: Option<DawStatus>,
    pub(crate) discord_connected: bool,
    window_id: Option<window::Id>,
    daw_scanner: Option<DawScanner>,
    discord: DiscordManager,
}

fn save_or_warn(settings: &AppSettings) {
    if let Err(error) = settings.save() {
        warn!("Couldn't save settings: {error}");
    }
}

/// Initialize application state: load settings, ensure daws.json, create scanner.
pub(crate) fn boot() -> (AppState, Task<Message>) {
    let config_path = match ensure_daw_config() {
        Ok(path) => Some(path),

        Err(error) => {
            warn!("Couldn't initialize daws.json: {error}");
            None
        }
    };

    let settings = AppSettings::load();
    let update_interval_input = settings.update_interval.to_string();
    let daw_scanner = config_path.and_then(|path| {
        crate::daw::load_configs(&path)
            .map(DawScanner::new)
            .map_err(|error| warn!("Couldn't load daws.json: {error}"))
            .ok()
    });

    (
        AppState {
            settings,
            update_interval_input,
            update_interval_error: None,
            show_interval_modal: false,
            daw_status: None,
            discord_connected: false,
            window_id: None,
            daw_scanner,
            discord: DiscordManager::default(),
        },
        Task::none(),
    )
}

/// Central message handler: UI toggles, settings persistence, DAW polling, Discord sync.
pub(crate) fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::CloseRequested(window_id) => {
            if state.settings.close_to_tray {
                window::set_mode(window_id, window::Mode::Hidden)
            } else {
                window::close(window_id)
            }
        }

        Message::WindowOpened(window_id) => {
            state.window_id = Some(window_id);
            Task::none()
        }

        Message::TrayShow => {
            let Some(window_id) = state.window_id else {
                return Task::none();
            };

            Task::batch(vec![
                window::set_mode(window_id, window::Mode::Windowed),
                window::gain_focus(window_id),
            ])
        }

        Message::TrayQuit => {
            let Some(window_id) = state.window_id else {
                return Task::none();
            };

            window::close(window_id)
        }

        Message::ToggleCloseToTray(value) => {
            state.settings.close_to_tray = value;
            save_or_warn(&state.settings);

            Task::none()
        }

        Message::ToggleHideProjectName(value) => {
            state.settings.hide_project_name = value;
            save_or_warn(&state.settings);

            crate::tray::send_tray_update(TrayUpdate::HideProjectName(value));
            Task::none()
        }

        Message::ToggleHideSystemUsage(value) => {
            state.settings.hide_system_usage = value;
            save_or_warn(&state.settings);

            crate::tray::send_tray_update(TrayUpdate::HideSystemUsage(value));
            Task::none()
        }

        Message::UpdateIntervalInput(value) => update_interval_input(state, &value),

        Message::OpenIntervalModal => {
            state.update_interval_input = state.settings.update_interval.to_string();
            state.update_interval_error = None;
            state.show_interval_modal = true;
            Task::none()
        }

        Message::CloseIntervalModal => {
            state.show_interval_modal = false;
            Task::none()
        }

        Message::ApplyInterval => apply_interval(state),

        Message::Tick => tick(state),
    }
}

fn update_interval_input(state: &mut AppState, value: &str) -> Task<Message> {
    state.update_interval_input = value.to_string();
    if value.trim().is_empty() {
        state.update_interval_error = Some("Interval must be a number".to_string());
    } else if let Ok(interval) = value.parse::<u64>() {
        match AppSettings::validate_update_interval(interval) {
            Ok(()) => state.update_interval_error = None,

            Err(error) => state.update_interval_error = Some(error.to_string()),
        }
    } else {
        state.update_interval_error = Some("Interval must be a number".to_string());
    }

    Task::none()
}

fn apply_interval(state: &mut AppState) -> Task<Message> {
    let Ok(interval) = state.update_interval_input.parse::<u64>() else {
        state.update_interval_error = Some("Interval must be a number".to_string());
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

fn tick(state: &mut AppState) -> Task<Message> {
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
        crate::tray::send_tray_update(TrayUpdate::DiscordConnected(connected));
    }

    Task::none()
}

pub(crate) fn subscription(state: &AppState) -> Subscription<Message> {
    let tick =
        time::every(Duration::from_millis(state.settings.update_interval)).map(|_| Message::Tick);
    Subscription::batch(vec![tray_subscription(), window_events(), tick])
}

fn window_events() -> Subscription<Message> {
    event::listen_with(|event, _status, window_id| match event {
        iced::Event::Window(window::Event::CloseRequested) => {
            Some(Message::CloseRequested(window_id))
        }

        iced::Event::Window(window::Event::Opened { .. }) => Some(Message::WindowOpened(window_id)),
        _ => None,
    })
}
