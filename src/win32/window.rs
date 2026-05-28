use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt as _;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Foundation::LPARAM;
use windows_sys::Win32::Foundation::TRUE;
use windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowTextW;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible;
use windows_sys::core::BOOL;

/// Find the longest visible window title for a PID.
pub(crate) fn window_title(pid: u32) -> String {
    struct State {
        target_pid: u32,
        titles: Vec<String>,
    }

    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = unsafe { &mut *(lparam as *mut State) };

        let mut process_id: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, &raw mut process_id) };

        if process_id != state.target_pid {
            return TRUE;
        }

        if unsafe { IsWindowVisible(hwnd) } == FALSE {
            return TRUE;
        }

        let text_len = unsafe { GetWindowTextLengthW(hwnd) };
        if text_len == 0 {
            return TRUE;
        }

        let mut buffer = vec![0u16; text_len as usize + 1];
        let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
        if len == 0 {
            return TRUE;
        }

        #[allow(clippy::indexing_slicing)]
        let title = OsString::from_wide(&buffer[..len as usize])
            .to_string_lossy()
            .to_string();
        if !title.trim().is_empty() {
            state.titles.push(title);
        }

        TRUE
    }

    let mut state = State {
        target_pid: pid,
        titles: Vec::new(),
    };

    unsafe {
        EnumWindows(Some(callback), &raw mut state as LPARAM);
    }

    // main window typically has the longest title
    state
        .titles
        .into_iter()
        .max_by_key(String::len)
        .unwrap_or_default()
}
