use tray_icon::Icon;
use tray_icon::TrayIcon;
use tray_icon::TrayIconBuilder;
use tray_icon::menu::CheckMenuItem;
use tray_icon::menu::Menu;
use tray_icon::menu::MenuId;
use tray_icon::menu::MenuItem;

use super::super::strings;
use super::icon::load_tray_icon;
use crate::error::TrayError;
use crate::settings::AppSettings;

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

pub(super) fn create_tray_icon() -> Result<(TrayIcon, TrayMenuIds), TrayError> {
    let settings = AppSettings::load();
    let menu = Menu::new();

    let hide_project = CheckMenuItem::new(
        strings::HIDE_PROJECT_NAME,
        true,
        settings.hide_project_name,
        None,
    );

    let hide_system = CheckMenuItem::new(
        strings::HIDE_SYSTEM_USAGE,
        true,
        settings.hide_system_usage,
        None,
    );

    let show = MenuItem::new(strings::TRAY_SHOW, true, None);
    let quit = MenuItem::new(strings::TRAY_QUIT, true, None);

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

    unsafe {
        let mut msg = zeroed();

        while PeekMessageW(&raw mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != FALSE {
            TranslateMessage(&raw const msg);
            DispatchMessageW(&raw const msg);
        }
    }
}

#[cfg(not(windows))]
pub(super) fn pump_windows_messages() {}
