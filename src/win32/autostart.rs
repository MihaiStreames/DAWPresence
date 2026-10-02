use tracing::trace;
use tracing::warn;
use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;

use super::registry;

const RUN_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "DAWPresence";
const AUTOSTART_FLAG: &str = "--autostart";
const LEGACY_AUTOSTART_FLAG: &str = "--minimized"; // versions before 3.0.5

// TODO: migrate legacy Run keys on boot
// if argv has LEGACY_AUTOSTART_FLAG the key is stale
// -> call set_enabled(true) to rewrite it with AUTOSTART_FLAG
// -> drop LEGACY_AUTOSTART_FLAG a few releases later
// --> users skipping would only see the window open on boot

/// Check if the app was launched by the Run key rather than by the user
pub(crate) fn is_autostart_launch() -> bool {
    std::env::args().any(|argument| argument == AUTOSTART_FLAG || argument == LEGACY_AUTOSTART_FLAG)
}

/// Check if auto-start is enabled by reading the HKCU Run key
pub(crate) fn is_enabled() -> bool {
    let exists = registry::value_exists(HKEY_CURRENT_USER, RUN_SUBKEY, VALUE_NAME);
    trace!("Registry auto-start check: {exists}");
    exists
}

/// Enable or disable auto-start by writing/deleting the HKCU Run key
pub(crate) fn set_enabled(enabled: bool) {
    if enabled {
        let Some(exe_path) = std::env::current_exe().ok() else {
            warn!("Couldn't determine executable path for auto-start");
            return;
        };

        // the flag lets startup apply the start in tray setting only to autostart launches
        let value = format!("\"{}\" {AUTOSTART_FLAG}", exe_path.display());
        trace!("Writing auto-start registry key: {value}");
        if !registry::set_sz(HKEY_CURRENT_USER, RUN_SUBKEY, VALUE_NAME, &value) {
            warn!("Couldn't write auto-start registry value");
        }
    } else {
        trace!("Removing auto-start registry key");
        registry::delete_value(HKEY_CURRENT_USER, RUN_SUBKEY, VALUE_NAME);
    }
}
