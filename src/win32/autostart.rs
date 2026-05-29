use tracing::trace;
use tracing::warn;
use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;

use super::registry;

const RUN_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "DAWPresence";

/// Check if auto-start is enabled by reading the HKCU Run key.
pub(crate) fn is_enabled() -> bool {
    let exists = registry::value_exists(HKEY_CURRENT_USER, RUN_SUBKEY, VALUE_NAME);
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

        // --minimized starts the app to tray without showing the window
        let value = format!("\"{}\" --minimized", exe_path.display());
        trace!("Writing auto-start registry key: {value}");
        if !registry::set_sz(HKEY_CURRENT_USER, RUN_SUBKEY, VALUE_NAME, &value) {
            warn!("Couldn't write auto-start registry value");
        }
    } else {
        trace!("Removing auto-start registry key");
        registry::delete_value(HKEY_CURRENT_USER, RUN_SUBKEY, VALUE_NAME);
    }
}
