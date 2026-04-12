//! System tray icon, menu, and event handling.

mod icon;
mod menu;

use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crossbeam_channel::RecvTimeoutError;
use iced::Subscription;
pub(crate) use icon::load_window_icon;
use tracing::debug;
use tracing::warn;
use tray_icon::menu::MenuEvent;

use self::icon::load_tray_icon;
use self::menu::TrayMenuIds;
use self::menu::create_tray_icon;
use self::menu::pump_windows_messages;
use crate::state::Message;

static TRAY_UPDATES: LazyLock<(
    std::sync::mpsc::Sender<TrayUpdate>,
    Mutex<std::sync::mpsc::Receiver<TrayUpdate>>,
)> = LazyLock::new(|| {
    let (sender, receiver) = std::sync::mpsc::channel();
    (sender, Mutex::new(receiver))
});

/// State changes pushed from the app thread to the tray thread.
pub(crate) enum TrayUpdate {
    HideProjectName(bool),
    HideSystemUsage(bool),
    DiscordConnected(bool),
}

/// Send a tray update to modify the tray menu
pub(crate) fn send_tray_update(update: TrayUpdate) {
    let _ = TRAY_UPDATES.0.send(update);
}

/// Bridge tray menu events into the app.
pub(crate) fn tray_subscription() -> Subscription<Message> {
    Subscription::run(|| {
        iced::stream::channel::<Message>(
            100,
            |output: iced::futures::channel::mpsc::Sender<Message>| async move {
                let shutdown = Arc::new(AtomicBool::new(false));
                let shutdown_clone = Arc::clone(&shutdown);

                std::thread::spawn(move || {
                    run_tray_handling(output, &shutdown_clone);
                });

                // when iced drops this future, the guard signals the tray thread to exit
                let _guard = ShutdownGuard(shutdown);
                iced::futures::future::pending::<()>().await;
            },
        )
    })
}

/// Sets the shutdown flag on drop so the tray thread exits cleanly.
struct ShutdownGuard(Arc<AtomicBool>);

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// Run tray icon handling in a separate thread.
fn run_tray_handling(
    mut output: iced::futures::channel::mpsc::Sender<Message>,
    shutdown: &AtomicBool,
) {
    let (tray_icon, menu_items) = match create_tray_icon() {
        Ok(tray) => tray,

        Err(error) => {
            warn!("Couldn't create tray icon: {error}");
            return;
        }
    };

    let receiver = MenuEvent::receiver().clone();
    while !shutdown.load(Ordering::Relaxed) {
        drain_tray_updates(&menu_items, &tray_icon);
        pump_windows_messages();

        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                if handle_tray_event(&menu_items, &mut output, &event) {
                    break;
                }
            }

            Err(RecvTimeoutError::Timeout) => {}

            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    drop(tray_icon);
}

/// Handle tray menu events and return true if the tray loop should exit
fn handle_tray_event(
    menu_items: &TrayMenuIds,
    output: &mut iced::futures::channel::mpsc::Sender<Message>,
    event: &tray_icon::menu::MenuEvent,
) -> bool {
    if event.id() == &menu_items.show {
        debug!("Tray: show requested");
        if output.try_send(Message::TrayShow).is_err() {
            warn!("Tray channel closed, exiting tray loop");
            return true;
        }

        return false;
    }

    if event.id() == &menu_items.quit {
        debug!("Tray: quit requested");
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
fn drain_tray_updates(menu_items: &TrayMenuIds, tray_icon: &tray_icon::TrayIcon) {
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
                debug!("Tray: discord connected = {connected}");
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
