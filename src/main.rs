// prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod daw;
mod discord;
mod settings;
mod ui;

use iced::{event, window, Subscription, Task};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::settings::AppSettings;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    CloseRequested(window::Id),
    WindowOpened(window::Id),
    TrayShow,
    TrayQuit,
    ToggleCloseToTray(bool),
    SelectPanel(Panel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Panel {
    Home,
    Settings,
}

pub(crate) struct AppState {
    pub(crate) settings: AppSettings,
    pub(crate) active_panel: Panel,
    window_id: Option<window::Id>,
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("DAWPRESENCE_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("DAWPresence v{} starting up", env!("CARGO_PKG_VERSION"));

    let window_icon = load_window_icon().ok();

    iced::application(boot, update, view)
        .subscription(subscription)
        .window(window::Settings {
            resizable: true,
            icon: window_icon,
            exit_on_close_request: false,
            ..window::Settings::default()
        })
        .run()
}

/// Load settings and initialize state
fn boot() -> (AppState, Task<Message>) {
    (
        AppState {
            settings: AppSettings::load(),
            active_panel: Panel::Home,
            window_id: None,
        },
        Task::none(),
    )
}

/// Update state and dispatch side effects
fn update(state: &mut AppState, message: Message) -> Task<Message> {
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
        Message::ToggleCloseToTray(close_to_tray) => {
            state.settings.close_to_tray = close_to_tray;
            if let Err(error) = state.settings.save() {
                warn!("Couldn't save settings: {error}");
            }
            Task::none()
        }
        Message::SelectPanel(panel) => {
            state.active_panel = panel;
            Task::none()
        }
    }
}

/// Render the main window content
fn view(state: &AppState) -> iced::Element<'_, Message> {
    ui::view(state)
}

/// Subscribe to tray and window events
fn subscription(_state: &AppState) -> Subscription<Message> {
    Subscription::batch(vec![tray_subscription(), window_events()])
}

/// Forward window close/open events into the app
fn window_events() -> Subscription<Message> {
    event::listen_with(|event, _status, window_id| match event {
        iced::Event::Window(window::Event::CloseRequested) => {
            Some(Message::CloseRequested(window_id))
        }
        iced::Event::Window(window::Event::Opened { .. }) => Some(Message::WindowOpened(window_id)),
        _ => None,
    })
}

#[derive(Clone, Copy, Debug)]
enum TrayAction {
    Show,
    Quit,
}

struct TrayMenuIds {
    show: MenuId,
    quit: MenuId,
}

/// Bridge tray menu events into the app
fn tray_subscription() -> Subscription<Message> {
    Subscription::run(|| {
        iced::stream::channel::<Message>(
            100,
            |output: iced::futures::channel::mpsc::Sender<Message>| async move {
                std::thread::spawn(move || {
                    #[cfg(target_os = "linux")]
                    {
                        run_tray_linux(output);
                    }

                    #[cfg(not(target_os = "linux"))]
                    {
                        run_tray_generic(output);
                    }
                });

                iced::futures::future::pending::<()>().await;
            },
        )
    })
}

/// Run tray icon handling for non-Linux platforms
/// Uses its own thread for event handling
#[cfg(not(target_os = "linux"))]
fn run_tray_generic(mut output: iced::futures::channel::mpsc::Sender<Message>) {
    let (tray_icon, menu_ids) = match create_tray_icon() {
        Ok(tray) => tray,
        Err(error) => {
            warn!("Couldn't create tray icon: {error}");
            return;
        }
    };

    let receiver = MenuEvent::receiver().clone();
    loop {
        let Ok(event) = receiver.recv() else {
            break;
        };
        let action = if event.id() == &menu_ids.show {
            Some(TrayAction::Show)
        } else if event.id() == &menu_ids.quit {
            Some(TrayAction::Quit)
        } else {
            None
        };

        if let Some(action) = action {
            let message = match action {
                TrayAction::Show => Message::TrayShow,
                TrayAction::Quit => Message::TrayQuit,
            };

            if output.try_send(message).is_err() {
                break;
            }

            if matches!(action, TrayAction::Quit) {
                break;
            }
        }
    }

    drop(tray_icon);
}

/// Run tray icon handling for Linux
/// This requires GTK's main loop to be run on the main thread
#[cfg(target_os = "linux")]
fn run_tray_linux(output: iced::futures::channel::mpsc::Sender<Message>) {
    if let Err(error) = gtk::init() {
        warn!("Couldn't init GTK for tray icon: {error}");
        return;
    }

    let (tray_icon, menu_ids) = match create_tray_icon() {
        Ok(tray) => tray,
        Err(error) => {
            warn!("Couldn't create tray icon: {error}");
            return;
        }
    };

    let receiver = MenuEvent::receiver().clone();
    std::thread::spawn(move || {
        let mut output = output;
        loop {
            let Ok(event) = receiver.recv() else {
                break;
            };
            let action = if event.id() == &menu_ids.show {
                Some(TrayAction::Show)
            } else if event.id() == &menu_ids.quit {
                Some(TrayAction::Quit)
            } else {
                None
            };

            if let Some(action) = action {
                let message = match action {
                    TrayAction::Show => Message::TrayShow,
                    TrayAction::Quit => Message::TrayQuit,
                };

                if output.try_send(message).is_err() {
                    break;
                }

                if matches!(action, TrayAction::Quit) {
                    gtk::glib::MainContext::default().invoke(gtk::main_quit);
                    break;
                }
            }
        }
    });

    gtk::main();
    drop(tray_icon);
}

/// Build the tray icon and return menu item ids
fn create_tray_icon() -> Result<(TrayIcon, TrayMenuIds), String> {
    let menu = Menu::new();
    let show = MenuItem::new("Show", true, None);
    let quit = MenuItem::new("Quit", true, None);
    menu.append(&show).map_err(|error| error.to_string())?;
    menu.append(&quit).map_err(|error| error.to_string())?;

    let icon = load_tray_icon()?;
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("DAWPresence")
        .with_icon(icon)
        .build()
        .map_err(|error| error.to_string())?;

    Ok((
        tray_icon,
        TrayMenuIds {
            show: show.id().clone(),
            quit: quit.id().clone(),
        },
    ))
}

/// Load the tray icon from embedded assets
fn load_tray_icon() -> Result<Icon, String> {
    let (rgba, width, height) = load_icon_rgba()?;
    Icon::from_rgba(rgba, width, height).map_err(|error| error.to_string())
}

/// Load the window icon from embedded assets
fn load_window_icon() -> Result<window::Icon, String> {
    let (rgba, width, height) = load_icon_rgba()?;
    window::icon::from_rgba(rgba, width, height).map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
const ICON_DATA: &[u8] = include_bytes!("assets/red.ico");
#[cfg(not(target_os = "windows"))]
const ICON_DATA: &[u8] = include_bytes!("assets/red.png");

#[cfg(target_os = "windows")]
const ICON_FORMAT: image::ImageFormat = image::ImageFormat::Ico;
#[cfg(not(target_os = "windows"))]
const ICON_FORMAT: image::ImageFormat = image::ImageFormat::Png;

/// Decode the selected icon asset into RGBA pixels
fn load_icon_rgba() -> Result<(Vec<u8>, u32, u32), String> {
    let image = image::load_from_memory_with_format(ICON_DATA, ICON_FORMAT)
        .map_err(|error| error.to_string())?
        .into_rgba8();
    Ok((image.to_vec(), image.width(), image.height()))
}
