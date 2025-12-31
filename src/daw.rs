use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};
use sysinfo::System;
use tracing::{debug, error, trace};

/// DAW configuration loaded from daws.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DawConfig {
    #[serde(rename = "ProcessName")]
    pub process_name: String,
    #[serde(rename = "DisplayText")]
    pub display_text: String,
    #[serde(rename = "TitleRegex")]
    pub title_regex: String,
    #[serde(rename = "ClientID")]
    pub client_id: String,
    #[serde(rename = "HideVersion")]
    #[serde(default)]
    pub hide_version: bool,
}

/// Current state of a detected DAW
#[derive(Debug, Clone, Default)]
pub struct DawStatus {
    pub is_running: bool,
    pub display_name: String,
    pub project_name: String,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub version: String,
    pub pid: u32,
    pub client_id: String,
    pub hide_version: bool,
}

impl DawStatus {
    /// Format CPU usage for display (e.g., "12.34%")
    pub fn cpu_usage_str(&self) -> String {
        if self.is_running {
            format!("{:.2}%", self.cpu_usage)
        } else {
            "Undefined".to_string()
        }
    }

    /// Format RAM usage for display (e.g., "1024MB")
    pub fn ram_usage_str(&self) -> String {
        if self.is_running {
            format!("{}MB", self.memory_mb)
        } else {
            "Undefined".to_string()
        }
    }
}

/// Monitors system processes for running DAWs
pub struct DawMonitor {
    configs: Vec<DawConfig>,
    system: System,
}

fn normalize_process_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .strip_suffix(".exe")
        .unwrap_or(name.trim())
        .to_lowercase()
}

impl DawMonitor {
    /// Create a new monitor with the given DAW configs
    pub fn new(configs: Vec<DawConfig>) -> Self {
        debug!("Loaded {} DAW configs", configs.len());
        Self {
            configs,
            system: System::new_all(),
        }
    }

    /// Load DAW configs from a JSON file
    pub fn load_configs(path: &Path) -> Result<Vec<DawConfig>, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Couldn't read daws.json: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("Couldn't parse daws.json: {e}"))
    }

    fn cpu_count(&self) -> usize {
        self.system.cpus().len().max(1)
    }

    /// Scan for running DAWs and return the first match
    pub fn scan(&mut self, hide_project_name: bool) -> Option<DawStatus> {
        self.system.refresh_all();

        for config in &self.configs {
            for (pid, process) in self.system.processes() {
                let process_name = normalize_process_name(&process.name().to_string_lossy());
                let config_name = normalize_process_name(&config.process_name);

                if process_name == config_name {
                    let window_title = get_window_title(*pid);
                    let project_name = if hide_project_name {
                        "(hidden)".to_string()
                    } else {
                        extract_project_name(&window_title, &config.title_regex)
                    };
                    let version = get_process_version(process.exe());

                    let cpu_count = self.cpu_count() as f32;
                    let normalized_cpu = process.cpu_usage() / cpu_count;
                    let memory_mb = process.memory() / (1024 * 1024);

                    trace!(
                        "Found {} (PID {}): {}MB RAM, {:.1}% CPU",
                        config.display_text,
                        pid.as_u32(),
                        memory_mb,
                        normalized_cpu
                    );

                    return Some(DawStatus {
                        is_running: true,
                        display_name: config.display_text.clone(),
                        project_name,
                        cpu_usage: normalized_cpu,
                        memory_mb,
                        version,
                        pid: pid.as_u32(),
                        client_id: config.client_id.clone(),
                        hide_version: config.hide_version,
                    });
                }
            }
        }

        None
    }
}

fn extract_project_name(title: &str, regex_pattern: &str) -> String {
    if title.is_empty() {
        return "None".to_string();
    }

    let Ok(re) = Regex::new(regex_pattern) else {
        error!("Invalid regex pattern: {}", regex_pattern);
        return "None".to_string();
    };

    let Some(captures) = re.captures(title) else {
        return "None".to_string();
    };

    captures
        .get(1)
        .or_else(|| captures.get(0))
        .map(|m| m.as_str().trim())
        .map(|s| s.trim_end_matches('*').trim()) // strip unsaved indicator
        .map(|s| if s.is_empty() { "Untitled" } else { s })
        .map(String::from)
        .unwrap_or_else(|| "None".to_string())
}

#[cfg(windows)]
fn get_process_version(exe_path: Option<&std::path::Path>) -> String {
    use std::os::windows::ffi::OsStrExt;

    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };

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

    let version_query = format!(
        "\\StringFileInfo\\{:04X}{:04X}\\ProductVersion",
        lang, codepage
    );
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

#[cfg(not(windows))]
fn get_process_version(_exe_path: Option<&std::path::Path>) -> String {
    "0.0.0".to_string()
}

#[cfg(windows)]
fn get_window_title(pid: sysinfo::Pid) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowEnabled, IsWindowVisible,
    };

    struct SearchState {
        target_pid: u32,
        result: Option<String>,
    }

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if lparam.0 == 0 {
            return BOOL(1);
        }

        let state = &mut *(lparam.0 as *mut SearchState);

        if !IsWindowVisible(hwnd).as_bool() || !IsWindowEnabled(hwnd).as_bool() {
            return BOOL(1);
        }

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id != state.target_pid {
            return BOOL(1);
        }

        let mut buffer = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buffer);
        if len == 0 {
            return BOOL(1);
        }

        let title = OsString::from_wide(&buffer[..len as usize])
            .to_string_lossy()
            .to_string();
        if title.trim().is_empty() {
            return BOOL(1);
        }

        state.result = Some(title);
        BOOL(0)
    }

    let mut state = SearchState {
        target_pid: pid.as_u32(),
        result: None,
    };

    unsafe {
        let _ = EnumWindows(Some(enum_callback), LPARAM(&mut state as *mut _ as isize));
    }

    state.result.unwrap_or_default()
}

#[cfg(not(windows))]
fn get_window_title(_pid: sysinfo::Pid) -> String {
    // TODO: implement for X11/Wayland
    String::new()
}
