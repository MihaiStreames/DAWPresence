use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::HKEY;
use windows_sys::Win32::System::Registry::KEY_READ;
use windows_sys::Win32::System::Registry::KEY_WRITE;
use windows_sys::Win32::System::Registry::REG_DWORD;
use windows_sys::Win32::System::Registry::REG_SZ;
use windows_sys::Win32::System::Registry::RegDeleteValueW;
use windows_sys::Win32::System::Registry::RegOpenKeyExW;
use windows_sys::Win32::System::Registry::RegQueryValueExW;
use windows_sys::Win32::System::Registry::RegSetValueExW;

use super::handle::OwnedKey;
use super::to_wide_null;

fn open_key(hive: HKEY, subkey: &str, access: u32) -> Option<OwnedKey> {
    let subkey = to_wide_null(subkey);
    let mut key = std::ptr::null_mut();
    let result = unsafe { RegOpenKeyExW(hive, subkey.as_ptr(), 0, access, &raw mut key) };
    (result == ERROR_SUCCESS).then_some(OwnedKey(key))
}

/// Check if a named value exists under the given key.
pub(crate) fn value_exists(hive: HKEY, subkey: &str, name: &str) -> bool {
    let Some(key) = open_key(hive, subkey, KEY_READ) else {
        return false;
    };

    let name = to_wide_null(name);

    let result = unsafe {
        RegQueryValueExW(
            key.0,
            name.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    result == ERROR_SUCCESS
}

/// Read a [`REG_DWORD`] value. Returns `None` if the key/value is missing or the type is wrong.
#[expect(dead_code)]
pub(crate) fn read_dword(hive: HKEY, subkey: &str, name: &str) -> Option<u32> {
    let key = open_key(hive, subkey, KEY_READ)?;
    let name = to_wide_null(name);

    let mut value: u32 = 0;
    let mut size = size_of::<u32>() as u32;
    let mut kind: u32 = 0;

    let result = unsafe {
        RegQueryValueExW(
            key.0,
            name.as_ptr(),
            std::ptr::null_mut(),
            &raw mut kind,
            (&raw mut value).cast(),
            &raw mut size,
        )
    };
    (result == ERROR_SUCCESS && kind == REG_DWORD).then_some(value)
}

/// Write a [`REG_SZ`] (string) value. Returns `false` if the key cannot be opened or write fails.
pub(crate) fn set_sz(hive: HKEY, subkey: &str, name: &str, value: &str) -> bool {
    let Some(key) = open_key(hive, subkey, KEY_WRITE) else {
        return false;
    };

    let name = to_wide_null(name);
    let value = to_wide_null(value);
    let byte_len = (value.len() * 2) as u32;

    let result = unsafe {
        RegSetValueExW(
            key.0,
            name.as_ptr(),
            0,
            REG_SZ,
            value.as_ptr().cast(),
            byte_len,
        )
    };
    result == ERROR_SUCCESS
}

/// Delete a named value. No-op if the key or value does not exist.
pub(crate) fn delete_value(hive: HKEY, subkey: &str, name: &str) {
    let Some(key) = open_key(hive, subkey, KEY_WRITE) else {
        return;
    };

    let name = to_wide_null(name);
    unsafe { RegDeleteValueW(key.0, name.as_ptr()) };
}
