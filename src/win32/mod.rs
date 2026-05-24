pub(crate) mod autostart;
pub(crate) mod handle;
pub(crate) mod process;
pub(crate) mod single_instance;
pub(crate) mod version;
pub(crate) mod watcher;
pub(crate) mod window;

/// Encode a string as null-terminated UTF-16 for Win32 APIs.
pub(super) fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
