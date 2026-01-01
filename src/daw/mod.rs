use std::path::{Path, PathBuf};
use std::thread;

use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};
use tracing::{debug, error, trace};

use crate::daw::windows::{get_process_version, get_window_title};

mod windows;

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
            let memory_kb = self.memory_mb.saturating_mul(1024);
            if memory_kb >= 1024 * 1024 {
                let memory_gb = memory_kb as f64 / (1024.0 * 1024.0);
                format!("{memory_gb:.2}GB")
            } else if memory_kb >= 1024 {
                format!("{}MB", self.memory_mb)
            } else {
                format!("{memory_kb}KB")
            }
        } else {
            "Undefined".to_string()
        }
    }
}

/// Monitors system processes for running DAWs
pub struct DawMonitor {
    configs: Vec<DawConfig>,
    system: System,
    cpu_count: usize,
}

/// Normalize process names for comparison
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
        let cpu_count = thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        Self {
            configs,
            system: System::new_with_specifics(RefreshKind::nothing()),
            cpu_count,
        }
    }

    /// Load DAW configs from a JSON file
    pub fn load_configs(path: &Path) -> Result<Vec<DawConfig>, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Couldn't read daws.json: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("Couldn't parse daws.json: {e}"))
    }

    /// Scan for running DAWs and return the first match
    pub fn scan(&mut self, hide_project_name: bool) -> Option<DawStatus> {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .with_exe(UpdateKind::OnlyIfNotSet)
                .without_tasks(),
        );

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

                    let normalized_cpu = process.cpu_usage() / self.cpu_count as f32;
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
                        client_id: config.client_id.clone(),
                        hide_version: config.hide_version,
                    });
                }
            }
        }

        None
    }
}

/// Ensure a bundled daws.json exists in the config directory
pub fn ensure_daw_config() -> Result<PathBuf, String> {
    let config_path =
        confy::get_configuration_file_path("dawpresence", None).map_err(|e| e.to_string())?;
    let config_dir = config_path
        .parent()
        .ok_or_else(|| "Couldn't resolve config directory".to_string())?;
    let daws_path = config_dir.join("daws.json");

    if !daws_path.exists() {
        std::fs::create_dir_all(config_dir)
            .map_err(|e| format!("Couldn't create config directory: {e}"))?;
        std::fs::write(&daws_path, include_bytes!("../../daws.json"))
            .map_err(|e| format!("Couldn't write daws.json: {e}"))?;
        debug!("Copied bundled daws.json to {}", daws_path.display());
    }

    Ok(daws_path)
}

/// Extract the project name from a window title using regex
fn extract_project_name(title: &str, regex_pattern: &str) -> String {
    if title.is_empty() {
        return "None".to_string();
    }

    let Ok(re) = Regex::new(regex_pattern) else {
        error!("Invalid regex pattern: {}", regex_pattern);
        return "None".to_string();
    };

    let Ok(Some(captures)) = re.captures(title) else {
        return "None".to_string();
    };

    captures
        .get(1)
        .or_else(|| captures.get(0))
        .map(|m| m.as_str().trim())
        .map(|s| s.trim_end_matches('*').trim())
        .map(|s| if s.is_empty() { "Untitled" } else { s })
        .map_or_else(|| "None".to_string(), String::from)
}
