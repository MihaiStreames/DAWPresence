use regex::Regex;
use serde::{Deserialize, Serialize};
use sysinfo::System;

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
    pub hide_version: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DawStatus {
    pub is_running: bool,
    pub display_name: String,
    pub project_name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub client_id: String,
    pub hide_version: bool,
}

pub struct DawMonitor {
    configs: Vec<DawConfig>,
    system: System,
}

impl DawMonitor {
    pub fn new(configs: Vec<DawConfig>) -> Self {
        Self {
            configs,
            system: System::new_all(),
        }
    }

    pub fn get_running_daw(&mut self) -> Option<DawStatus> {
        self.system.refresh_all();

        for config in &self.configs {
            for (pid, process) in self.system.processes() {
                let process_name = process.name().to_string_lossy().to_lowercase();
                let config_name = config.process_name.to_lowercase();

                let matches = process_name == config_name
                    || process_name == format!("{}.exe", config_name)
                    || process_name.starts_with(&config_name);

                if matches {
                    let window_title = get_window_title(*pid);
                    let project_name = extract_project_name(&window_title, &config.title_regex);

                    return Some(DawStatus {
                        is_running: true,
                        display_name: config.display_text.clone(),
                        project_name,
                        cpu_usage: process.cpu_usage(),
                        memory_usage: process.memory(),
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
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| "None".to_string())
}

#[cfg(windows)]
fn get_window_title(pid: sysinfo::Pid) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId,
    };

    static mut RESULT: Option<String> = None;
    static mut TARGET_PID: u32 = 0;

    unsafe extern "system" fn enum_callback(hwnd: HWND, _: LPARAM) -> BOOL {
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id == TARGET_PID {
            let mut buffer = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut buffer);
            if len > 0 {
                let title = OsString::from_wide(&buffer[..len as usize])
                    .to_string_lossy()
                    .to_string();
                if !title.is_empty() {
                    RESULT = Some(title);
                    return BOOL(0);
                }
            }
        }
        BOOL(1)
    }

    unsafe {
        RESULT = None;
        TARGET_PID = pid.as_u32();
        let _ = EnumWindows(Some(enum_callback), LPARAM(0));
        RESULT.take().unwrap_or_default()
    }
}

#[cfg(not(windows))]
fn get_window_title(_pid: sysinfo::Pid) -> String {
    // TODO: implement for X11/Wayland
    String::new()
}

pub fn format_memory(bytes: u64) -> String {
    let mb = bytes as f64 / 1024.0 / 1024.0;
    if mb >= 1024.0 {
        format!("{:.2} GB", mb / 1024.0)
    } else {
        format!("{:.0} MB", mb)
    }
}

pub fn format_cpu(usage: f32) -> String {
    format!("{:.1}%", usage)
}
