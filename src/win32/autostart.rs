use tracing::trace;
use tracing::warn;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;
use windows_sys::Win32::System::Registry::KEY_READ;
use windows_sys::Win32::System::Registry::KEY_WRITE;
use windows_sys::Win32::System::Registry::REG_SZ;
use windows_sys::Win32::System::Registry::RegCloseKey;
use windows_sys::Win32::System::Registry::RegDeleteValueW;
use windows_sys::Win32::System::Registry::RegOpenKeyExW;
use windows_sys::Win32::System::Registry::RegQueryValueExW;
use windows_sys::Win32::System::Registry::RegSetValueExW;

use super::to_wide_null;

const RUN_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "DAWPresence";

/// Check if auto-start is enabled by reading the HKCU Run key.
pub(crate) fn is_enabled() -> bool {
    let subkey = to_wide_null(RUN_SUBKEY);
    let name = to_wide_null(VALUE_NAME);
    let mut key = std::ptr::null_mut();

    // SAFETY: standard registry read, key closed on all paths
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_READ,
            &raw mut key,
        )
    };

    if result != ERROR_SUCCESS {
        return false;
    }

    let exists = unsafe {
        RegQueryValueExW(
            key,
            name.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    } == ERROR_SUCCESS;

    unsafe { RegCloseKey(key) };
    trace!("Registry auto-start check: {exists}");
    exists
}

/// Enable or disable auto-start by writing/deleting the HKCU Run key.
pub(crate) fn set_enabled(enabled: bool) {
    if enabled {
        let Some(exe_path) = std::env::current_exe().ok() else {
            warn!("Couldn't determine executable path for auto-start");
            return;
        };

        let value = format!("\"{}\" --minimized", exe_path.display());
        trace!("Writing auto-start registry key: {value}");
        write_run_value(&value);
    } else {
        trace!("Removing auto-start registry key");
        delete_run_value();
    }
}

fn write_run_value(exe_path: &str) {
    let subkey = to_wide_null(RUN_SUBKEY);
    let name = to_wide_null(VALUE_NAME);
    let value = to_wide_null(exe_path);
    let mut key = std::ptr::null_mut();

    // SAFETY: standard registry write, key closed on all paths
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_WRITE,
            &raw mut key,
        )
    };

    if result != ERROR_SUCCESS {
        warn!("Couldn't open Run registry key: error {result}");
        return;
    }

    let byte_len = (value.len() * 2) as u32;
    let result = unsafe {
        RegSetValueExW(
            key,
            name.as_ptr(),
            0,
            REG_SZ,
            value.as_ptr().cast(),
            byte_len,
        )
    };

    if result != ERROR_SUCCESS {
        warn!("Couldn't write auto-start registry value: error {result}");
    }

    unsafe { RegCloseKey(key) };
}

fn delete_run_value() {
    let subkey = to_wide_null(RUN_SUBKEY);
    let name = to_wide_null(VALUE_NAME);
    let mut key = std::ptr::null_mut();

    // SAFETY: standard registry write, key closed on all paths
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_WRITE,
            &raw mut key,
        )
    };

    if result != ERROR_SUCCESS {
        return;
    }

    unsafe { RegDeleteValueW(key, name.as_ptr()) };
    unsafe { RegCloseKey(key) };
}
