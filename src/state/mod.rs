mod handlers;

use std::time::Duration;

use iced::Subscription;
use iced::Task;
use iced::event;
use iced::futures::channel::mpsc::Sender;
use iced::futures::future;
use iced::time;
use iced::window;
use tracing::warn;

use crate::daw::DawScanner;
use crate::daw::DawStatus;
use crate::daw::ensure_daw_config;
use crate::discord::DiscordManager;
use crate::settings::AppSettings;
use crate::ui::tray::tray_subscription;
use crate::win32::autostart;
use crate::win32::single_instance;

/// Active page in the sidebar navigation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Page {
    #[default]
    Home,
    Settings,
}

/// Events flowing through the Iced update loop.
#[derive(Debug, Clone)]
pub(crate) enum Message {
    CloseRequested(window::Id),
    WindowOpened(window::Id),
    TrayShow,
    TrayQuit,
    NavigateTo(Page),
    ToggleAutoStart(bool),
    ToggleCloseToTray(bool),
    ToggleHideProjectName(bool),
    ToggleHideSystemUsage(bool),
    ToggleTimerMode,
    UpdateIntervalInput(String),
    ApplyInterval,
    Tick,
}

/// Root application state for the Iced MVU loop.
pub(crate) struct AppState {
    pub(crate) settings: AppSettings,
    pub(crate) active_page: Page,
    pub(crate) update_interval_input: String,
    pub(crate) update_interval_error: Option<String>,
    pub(crate) interval_applied: bool,
    pub(crate) daw_status: Option<DawStatus>,
    pub(crate) discord_connected: bool,
    pub(crate) auto_start_enabled: bool,
    window_id: Option<window::Id>,
    daw_scanner: Option<DawScanner>,
    discord: DiscordManager,
    last_project_name: Option<String>,
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

    let auto_start_enabled = autostart::is_enabled();

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
            active_page: Page::Home,
            update_interval_input,
            update_interval_error: None,
            interval_applied: false,
            daw_status: None,
            discord_connected: false,
            auto_start_enabled,
            window_id: None,
            daw_scanner,
            discord: DiscordManager::default(),
            last_project_name: None,
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
        Message::NavigateTo(page) => handlers::navigate_to(state, page),
        Message::ToggleAutoStart(v) => handlers::toggle_auto_start(state, v),
        Message::ToggleCloseToTray(v) => handlers::toggle_close_to_tray(state, v),
        Message::ToggleHideProjectName(v) => handlers::toggle_hide_project_name(state, v),
        Message::ToggleHideSystemUsage(v) => handlers::toggle_hide_system_usage(state, v),
        Message::ToggleTimerMode => handlers::toggle_timer_mode(state),
        Message::UpdateIntervalInput(v) => handlers::update_interval_input(state, &v),
        Message::ApplyInterval => handlers::apply_interval(state),
        Message::Tick => handlers::tick(state),
    }
}

/// Subscribe to tray events, window events, and periodic DAW polling.
pub(crate) fn subscription(state: &AppState) -> Subscription<Message> {
    let tick =
        time::every(Duration::from_millis(state.settings.update_interval)).map(|_| Message::Tick);
    Subscription::batch(vec![
        tray_subscription(),
        window_events(),
        tick,
        single_instance_subscription(),
    ])
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

fn single_instance_subscription() -> Subscription<Message> {
    Subscription::run(|| {
        iced::stream::channel(1, |mut output: Sender<Message>| async move {
            let Some(receiver) = single_instance::take_receiver() else {
                future::pending::<()>().await;
                return;
            };

            std::thread::Builder::new()
                .name("single-instance-show".into())
                .stack_size(64 * 1024)
                .spawn(move || {
                    while receiver.recv().is_ok() {
                        if output.try_send(Message::TrayShow).is_err() {
                            break;
                        }
                    }
                })
                .expect("couldn't spawn single-instance show thread");

            future::pending::<()>().await;
        })
    })
}
