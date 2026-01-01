#![allow(unsafe_code)] // FFI with Win32 APIs requires unsafe

use std::path::Path;

use sysinfo::Pid;
#[cfg(windows)]
use tracing::{debug, trace};

#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;
#[cfg(windows)]
use windows::core::{BOOL, PCWSTR};
#[cfg(windows)]
use windows::Win32::Foundation::{HWND, LPARAM};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
};

/// Fetch a Windows file version string from the executable metadata
#[cfg(windows)]
pub fn get_process_version(exe_path: Option<&Path>) -> String {
    let Some(path) = exe_path else {
        return "0.0.0".to_string();
    };

    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut handle: u32 = 0;
    let size = unsafe {
        GetFileVersionInfoSizeW(PCWSTR(path_wide.as_ptr()), Some(&mut handle as *mut u32))
    };
    if size == 0 {
        return "0.0.0".to_string();
    }

    let mut data = vec![0u8; size as usize];
    if unsafe {
        GetFileVersionInfoW(
            PCWSTR(path_wide.as_ptr()),
            Some(handle),
            size,
            data.as_mut_ptr() as *mut _,
        )
    }
    .is_err()
    {
        return "0.0.0".to_string();
    }

    let translation_query: Vec<u16> = "\\VarFileInfo\\Translation"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut translation_ptr: *mut core::ffi::c_void = std::ptr::null_mut();
    let mut translation_len: u32 = 0;
    let ok = unsafe {
        VerQueryValueW(
            data.as_ptr() as *const _,
            PCWSTR(translation_query.as_ptr()),
            &mut translation_ptr,
            &mut translation_len,
        )
    };

    if !ok.as_bool() || translation_ptr.is_null() || translation_len < 4 {
        return "0.0.0".to_string();
    }

    let data_start = data.as_ptr() as usize;
    let data_end = data_start + data.len();
    let translation_addr = translation_ptr as usize;
    if !(data_start..data_end).contains(&translation_addr) || translation_addr + 4 > data_end {
        return "0.0.0".to_string();
    }

    let translation = unsafe { std::slice::from_raw_parts(translation_ptr as *const u16, 2) };
    let lang = translation[0];
    let codepage = translation[1];

    let version_query = format!("\\StringFileInfo\\{lang:04X}{codepage:04X}\\ProductVersion");
    let version_query: Vec<u16> = version_query
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut version_ptr: *mut core::ffi::c_void = std::ptr::null_mut();
    let mut version_len: u32 = 0;
    let ok = unsafe {
        VerQueryValueW(
            data.as_ptr() as *const _,
            PCWSTR(version_query.as_ptr()),
            &mut version_ptr,
            &mut version_len,
        )
    };

    if !ok.as_bool() || version_ptr.is_null() {
        return "0.0.0".to_string();
    }

    let version_addr = version_ptr as usize;
    if !(data_start..data_end).contains(&version_addr) {
        return "0.0.0".to_string();
    }

    let version_len = version_len as usize;
    let max_len = (data_end - version_addr) / 2;
    let len = version_len.min(max_len).max(1);
    let version_wide = unsafe { std::slice::from_raw_parts(version_ptr as *const u16, len) };
    let len = version_wide
        .iter()
        .position(|c| *c == 0)
        .unwrap_or(version_wide.len());
    let version = String::from_utf16_lossy(&version_wide[..len])
        .trim()
        .to_string();

    if version.is_empty() {
        return "0.0.0".to_string();
    }

    version
}

/// Look up window titles for a PID on Windows, returning the longest one
/// (main windows typically have longer titles than toolbars/palettes)
#[cfg(windows)]
pub fn get_window_title(pid: Pid) -> String {
    struct SearchState {
        target_pid: u32,
        titles: Vec<String>,
    }

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if lparam.0 == 0 {
            return BOOL(1);
        }

        let state = &mut *(lparam.0 as *mut SearchState);

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id != state.target_pid {
            return BOOL(1);
        }

        // check visibility after PID match for better debugging
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        let text_len = GetWindowTextLengthW(hwnd);
        if text_len == 0 {
            return BOOL(1);
        }

        let mut buffer = vec![0u16; text_len as usize + 1];
        let len = GetWindowTextW(hwnd, &mut buffer);
        if len == 0 {
            return BOOL(1);
        }

        let title = OsString::from_wide(&buffer[..len as usize])
            .to_string_lossy()
            .to_string();
        if !title.trim().is_empty() {
            state.titles.push(title);
        }

        BOOL(1) // continue enumeration to find all windows
    }

    let mut state = SearchState {
        target_pid: pid.as_u32(),
        titles: Vec::new(),
    };

    unsafe {
        let _ = EnumWindows(Some(enum_callback), LPARAM(&mut state as *mut _ as isize));
    }

    if state.titles.is_empty() {
        debug!("no window titles found for PID {}", pid.as_u32());
        return String::new();
    }

    trace!(
        "found {} window(s) for PID {}: {:?}",
        state.titles.len(),
        pid.as_u32(),
        state.titles
    );

    // return the longest title (main window usually has project name, toolbars are short)
    state
        .titles
        .into_iter()
        .max_by_key(String::len)
        .unwrap_or_default()
}

/// Return a default version on unsupported platforms
#[cfg(not(windows))]
pub fn get_process_version(_exe_path: Option<&Path>) -> String {
    "0.0.0".to_string()
}

/// Return empty window titles on unsupported platforms
#[cfg(not(windows))]
pub fn get_window_title(_pid: Pid) -> String {
    String::new()
}
