//! Window title extraction via EnumWindows.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

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
pub(in crate::daw) fn window_title(pid: u32) -> String {
    struct State {
        target_pid: u32,
        titles: Vec<String>,
    }

    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = unsafe { &mut *(lparam as *mut State) };

        let mut process_id: u32 = 0;
        // SAFETY: hwnd is from EnumWindows, process_id is stack-local
        unsafe { GetWindowThreadProcessId(hwnd, &mut process_id) };

        if process_id != state.target_pid {
            return TRUE;
        }

        // SAFETY: hwnd is valid from EnumWindows
        if unsafe { IsWindowVisible(hwnd) } == FALSE {
            return TRUE;
        }

        // SAFETY: hwnd is valid
        let text_len = unsafe { GetWindowTextLengthW(hwnd) };
        if text_len == 0 {
            return TRUE;
        }

        let mut buffer = vec![0u16; text_len as usize + 1];
        // SAFETY: buffer is correctly sized for the title
        let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
        if len == 0 {
            return TRUE;
        }

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

    // SAFETY: state lives on stack for duration of enumeration, callback only borrows it
    unsafe {
        EnumWindows(Some(callback), &mut state as *mut State as LPARAM);
    }

    // main window typically has the longest title
    state
        .titles
        .into_iter()
        .max_by_key(String::len)
        .unwrap_or_default()
}
