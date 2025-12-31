use std::path::Path;

use sysinfo::Pid;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(windows)]
mod windows;
#[cfg(all(not(windows), not(target_os = "linux")))]
mod fallback;

#[cfg(target_os = "linux")]
pub fn get_process_version(exe_path: Option<&Path>) -> String {
    linux::get_process_version(exe_path)
}

#[cfg(windows)]
pub fn get_process_version(exe_path: Option<&Path>) -> String {
    windows::get_process_version(exe_path)
}

#[cfg(all(not(windows), not(target_os = "linux")))]
pub fn get_process_version(exe_path: Option<&Path>) -> String {
    fallback::get_process_version(exe_path)
}

#[cfg(target_os = "linux")]
pub fn get_window_title(pid: Pid) -> String {
    linux::get_window_title(pid)
}

#[cfg(windows)]
pub fn get_window_title(pid: Pid) -> String {
    windows::get_window_title(pid)
}

#[cfg(all(not(windows), not(target_os = "linux")))]
pub fn get_window_title(pid: Pid) -> String {
    fallback::get_window_title(pid)
}
