//! Tray menu creation and Windows message pump.

use tray_icon::Icon;
use tray_icon::TrayIcon;
use tray_icon::TrayIconBuilder;
use tray_icon::menu::CheckMenuItem;
use tray_icon::menu::Menu;
use tray_icon::menu::MenuId;
use tray_icon::menu::MenuItem;

use crate::error::TrayError;
use crate::settings::AppSettings;
use crate::tray::icon::load_tray_icon;

/// Map any Display error into `TrayError::CreateFailed`.
fn create_err(error: impl std::fmt::Display) -> TrayError {
    TrayError::CreateFailed(error.to_string())
}

pub(super) struct TrayMenuIds {
    pub(super) show: MenuId,
    pub(super) quit: MenuId,
    pub(super) hide_project: CheckMenuItem,
    pub(super) hide_system: CheckMenuItem,
}

pub(super) fn create_tray_icon() -> Result<(TrayIcon, TrayMenuIds), crate::error::TrayError> {
    let settings = AppSettings::load();
    let menu = Menu::new();
    let hide_project =
        CheckMenuItem::new("Hide project name", true, settings.hide_project_name, None);
    let hide_system =
        CheckMenuItem::new("Hide system usage", true, settings.hide_system_usage, None);
    let show = MenuItem::new("Show", true, None);
    let quit = MenuItem::new("Quit", true, None);

    menu.append(&hide_project).map_err(create_err)?;
    menu.append(&hide_system).map_err(create_err)?;
    menu.append(&show).map_err(create_err)?;
    menu.append(&quit).map_err(create_err)?;

    let icon: Icon = load_tray_icon(false)?;
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("DAWPresence")
        .with_icon(icon)
        .build()
        .map_err(create_err)?;

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

/// Pump Windows messages to keep the tray icon responsive.
#[cfg(windows)]
#[allow(unsafe_code)]
pub(super) fn pump_windows_messages() {
    use std::mem::zeroed;

    use windows_sys::Win32::Foundation::FALSE;
    use windows_sys::Win32::UI::WindowsAndMessaging::DispatchMessageW;
    use windows_sys::Win32::UI::WindowsAndMessaging::PM_REMOVE;
    use windows_sys::Win32::UI::WindowsAndMessaging::PeekMessageW;
    use windows_sys::Win32::UI::WindowsAndMessaging::TranslateMessage;

    // SAFETY: msg is zeroed, PeekMessageW/TranslateMessage/DispatchMessageW are standard Win32
    unsafe {
        let mut msg = zeroed();

        while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != FALSE {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(not(windows))]
pub(super) fn pump_windows_messages() {}
