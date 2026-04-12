//! Application state machine.

mod handlers;

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
use crate::ui::tray::tray_subscription;

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
    OverlayClicked,
    ApplyInterval,
    Tick,
}

/// Root application state for the Iced MVU loop.
pub(crate) struct AppState {
    pub(crate) settings: AppSettings,
    pub(crate) update_interval_input: String,
    pub(crate) update_interval_error: Option<String>,
    pub(crate) modal_dismiss_warning: bool,
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
            modal_dismiss_warning: false,
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

/// Central message router: dispatches to handlers.
pub(crate) fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::CloseRequested(id) => handlers::close_requested(&state.settings, id),
        Message::WindowOpened(id) => handlers::window_opened(state, id),
        Message::TrayShow => handlers::tray_show(state.window_id),
        Message::TrayQuit => handlers::tray_quit(state.window_id),
        Message::ToggleCloseToTray(v) => handlers::toggle_close_to_tray(state, v),
        Message::ToggleHideProjectName(v) => handlers::toggle_hide_project_name(state, v),
        Message::ToggleHideSystemUsage(v) => handlers::toggle_hide_system_usage(state, v),
        Message::UpdateIntervalInput(v) => handlers::update_interval_input(state, &v),
        Message::OpenIntervalModal => handlers::open_interval_modal(state),
        Message::CloseIntervalModal => handlers::close_interval_modal(state),
        Message::OverlayClicked => handlers::overlay_clicked(state),
        Message::ApplyInterval => handlers::apply_interval(state),
        Message::Tick => handlers::tick(state),
    }
}

/// Subscribe to tray events, window events, and periodic DAW polling.
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
