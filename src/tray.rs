use iced::{window, Subscription};
use std::sync::{LazyLock, Mutex};
#[cfg(not(target_os = "linux"))]
use std::time::Duration;
use tracing::warn;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::settings::AppSettings;
use crate::Message;

static TRAY_UPDATES: LazyLock<(
    std::sync::mpsc::Sender<TrayUpdate>,
    Mutex<std::sync::mpsc::Receiver<TrayUpdate>>,
)> = LazyLock::new(|| {
    let (sender, receiver) = std::sync::mpsc::channel();
    (sender, Mutex::new(receiver))
});

pub(crate) enum TrayUpdate {
    HideProjectName(bool),
    HideSystemUsage(bool),
    DiscordConnected(bool),
}

/// Send a tray update to modify the tray menu
pub(crate) fn send_tray_update(update: TrayUpdate) {
    let _ = TRAY_UPDATES.0.send(update);
}

struct TrayMenuIds {
    show: MenuId,
    quit: MenuId,
    hide_project: CheckMenuItem,
    hide_system: CheckMenuItem,
}

/// Bridge tray menu events into the app
pub(crate) fn tray_subscription() -> Subscription<Message> {
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
/// uses a blocking loop
#[cfg(not(target_os = "linux"))]
fn run_tray_generic(mut output: iced::futures::channel::mpsc::Sender<Message>) {
    let (tray_icon, menu_items) = match create_tray_icon() {
        Ok(tray) => tray,
        Err(error) => {
            warn!("Couldn't create tray icon: {error}");
            return;
        }
    };

    let receiver = MenuEvent::receiver().clone();
    loop {
        drain_tray_updates(&menu_items, &tray_icon);
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                if handle_tray_event(&menu_items, &mut output, event) {
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    drop(tray_icon);
}

/// Run tray icon handling for Linux
/// uses GTK's main loop
#[cfg(target_os = "linux")]
fn run_tray_linux(output: iced::futures::channel::mpsc::Sender<Message>) {
    if let Err(error) = gtk::init() {
        warn!("Couldn't init GTK for tray icon: {error}");
        return;
    }

    let (tray_icon, menu_items) = match create_tray_icon() {
        Ok(tray) => tray,
        Err(error) => {
            warn!("Couldn't create tray icon: {error}");
            return;
        }
    };

    let receiver = MenuEvent::receiver().clone();
    let mut output = output;
    let tray_icon_handle = tray_icon.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        drain_tray_updates(&menu_items, &tray_icon_handle);
        while let Ok(event) = receiver.try_recv() {
            if handle_tray_event(&menu_items, &mut output, event) {
                gtk::main_quit();
                return gtk::glib::ControlFlow::Break;
            }
        }
        gtk::glib::ControlFlow::Continue
    });

    gtk::main();
    drop(tray_icon);
}

/// Build the tray icon and return menu item ids
fn create_tray_icon() -> Result<(TrayIcon, TrayMenuIds), String> {
    let settings = AppSettings::load();
    let menu = Menu::new();
    let hide_project =
        CheckMenuItem::new("Hide project name", true, settings.hide_project_name, None);
    let hide_system =
        CheckMenuItem::new("Hide system usage", true, settings.hide_system_usage, None);
    let show = MenuItem::new("Show", true, None);
    let quit = MenuItem::new("Quit", true, None);
    menu.append(&hide_project)
        .map_err(|error| error.to_string())?;
    menu.append(&hide_system)
        .map_err(|error| error.to_string())?;
    menu.append(&show).map_err(|error| error.to_string())?;
    menu.append(&quit).map_err(|error| error.to_string())?;

    let icon = load_tray_icon(false)?;
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
            hide_project,
            hide_system,
        },
    ))
}

/// Handle tray menu events and return true if the tray loop should exit
fn handle_tray_event(
    menu_items: &TrayMenuIds,
    output: &mut iced::futures::channel::mpsc::Sender<Message>,
    event: tray_icon::menu::MenuEvent,
) -> bool {
    if event.id() == &menu_items.show {
        return output.try_send(Message::TrayShow).is_err();
    }

    if event.id() == &menu_items.quit {
        let _ = output.try_send(Message::TrayQuit);
        return true;
    }

    if event.id() == menu_items.hide_project.id() {
        let checked = menu_items.hide_project.is_checked();
        let _ = output.try_send(Message::ToggleHideProjectName(checked));
        return false;
    }

    if event.id() == menu_items.hide_system.id() {
        let checked = menu_items.hide_system.is_checked();
        let _ = output.try_send(Message::ToggleHideSystemUsage(checked));
        return false;
    }

    false
}

/// Apply any pending tray updates to the menu items
fn drain_tray_updates(menu_items: &TrayMenuIds, tray_icon: &TrayIcon) {
    let Ok(receiver) = TRAY_UPDATES.1.lock() else {
        return;
    };
    for update in receiver.try_iter() {
        match update {
            TrayUpdate::HideProjectName(checked) => {
                menu_items.hide_project.set_checked(checked);
            }
            TrayUpdate::HideSystemUsage(checked) => {
                menu_items.hide_system.set_checked(checked);
            }
            TrayUpdate::DiscordConnected(connected) => {
                let icon = match load_tray_icon(connected) {
                    Ok(icon) => icon,
                    Err(error) => {
                        warn!("Couldn't update tray icon: {error}");
                        continue;
                    }
                };
                if let Err(error) = tray_icon.set_icon(Some(icon)) {
                    warn!("Couldn't set tray icon: {error}");
                }
            }
        }
    }
}

/// Load the tray icon from embedded assets
fn load_tray_icon(connected: bool) -> Result<Icon, String> {
    let (rgba, width, height) = load_icon_rgba(connected)?;
    Icon::from_rgba(rgba, width, height).map_err(|error| error.to_string())
}

/// Load the window icon from embedded assets
pub(crate) fn load_window_icon() -> Result<window::Icon, String> {
    let (rgba, width, height) = load_icon_rgba(false)?;
    window::icon::from_rgba(rgba, width, height).map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
const ICON_RED_DATA: &[u8] = include_bytes!("assets/red.ico");
#[cfg(not(target_os = "windows"))]
const ICON_RED_DATA: &[u8] = include_bytes!("assets/red.png");

#[cfg(target_os = "windows")]
const ICON_GREEN_DATA: &[u8] = include_bytes!("assets/green.ico");
#[cfg(not(target_os = "windows"))]
const ICON_GREEN_DATA: &[u8] = include_bytes!("assets/green.png");

#[cfg(target_os = "windows")]
const ICON_FORMAT: image::ImageFormat = image::ImageFormat::Ico;
#[cfg(not(target_os = "windows"))]
const ICON_FORMAT: image::ImageFormat = image::ImageFormat::Png;

/// Decode the selected icon asset into RGBA pixels
fn load_icon_rgba(connected: bool) -> Result<(Vec<u8>, u32, u32), String> {
    let data = if connected {
        ICON_GREEN_DATA
    } else {
        ICON_RED_DATA
    };
    let image = image::load_from_memory_with_format(data, ICON_FORMAT)
        .map_err(|error| error.to_string())?
        .into_rgba8();
    Ok((image.to_vec(), image.width(), image.height()))
}
