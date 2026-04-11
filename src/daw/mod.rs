use std::path::Path;
use std::path::PathBuf;
use std::thread;

use fancy_regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use sysinfo::ProcessRefreshKind;
use sysinfo::ProcessesToUpdate;
use sysinfo::RefreshKind;
use sysinfo::System;
use sysinfo::UpdateKind;
use tracing::debug;
use tracing::error;
use tracing::trace;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use crate::daw::windows::get_process_version;
#[cfg(windows)]
use crate::daw::windows::get_window_title;

#[cfg(not(windows))]
mod unsupported;

#[cfg(not(windows))]
use crate::daw::unsupported::get_process_version;
#[cfg(not(windows))]
use crate::daw::unsupported::get_window_title;

/// Versioned wrapper for the daws.json config file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DawConfigFile {
    version: u32,
    daws: Vec<DawConfig>,
}

/// DAW configuration loaded from daws.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DawConfig {
    #[serde(rename = "ProcessName")]
    pub(crate) process_name: String,
    #[serde(rename = "DisplayText")]
    pub(crate) display_text: String,
    #[serde(rename = "TitleRegex")]
    pub(crate) title_regex: String,
    #[serde(rename = "ClientID")]
    pub(crate) client_id: String,
    #[serde(rename = "HideVersion")]
    #[serde(default)]
    pub(crate) hide_version: bool,
    /// Extra process name prefixes whose CPU/RAM should be aggregated
    /// (e.g. "BitwigAudioEngine" matches "BitwigAudioEngine-X64-AVX2.exe")
    #[serde(rename = "AdditionalProcessNames")]
    #[serde(default)]
    pub(crate) additional_process_names: Vec<String>,
}

/// Current state of a detected DAW
#[derive(Debug, Clone, Default)]
pub(crate) struct DawStatus {
    pub(crate) is_running: bool,
    pub(crate) display_name: String,
    pub(crate) project_name: String,
    pub(crate) cpu_usage: f32,
    pub(crate) memory_mb: u64,
    pub(crate) version: String,
    pub(crate) client_id: String,
    pub(crate) hide_version: bool,
}

impl DawStatus {
    /// Format CPU usage for display (e.g., "12.34%")
    pub(crate) fn cpu_usage_str(&self) -> String {
        if self.is_running {
            format!("{:.2}%", self.cpu_usage)
        } else {
            "Undefined".to_string()
        }
    }

    /// Format RAM usage for display (e.g., "1024MB")
    pub(crate) fn ram_usage_str(&self) -> String {
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
pub(crate) struct DawMonitor {
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
    pub(crate) fn new(configs: Vec<DawConfig>) -> Self {
        debug!("Loaded {} DAW configs", configs.len());
        let cpu_count = thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        Self {
            configs,
            system: System::new_with_specifics(RefreshKind::nothing()),
            cpu_count,
        }
    }

    /// Load DAW configs from a JSON file
    pub(crate) fn load_configs(path: &Path) -> Result<Vec<DawConfig>, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Couldn't read daws.json: {e}"))?;
        let file: DawConfigFile =
            serde_json::from_str(&content).map_err(|e| format!("Couldn't parse daws.json: {e}"))?;
        Ok(file.daws)
    }

    /// Scan for running DAWs and return the first matching config.
    /// Aggregates CPU/RAM across all processes matching the same DAW
    /// (e.g. Bitwig spawns multiple sound engine processes).
    pub(crate) fn scan(&mut self, hide_project_name: bool) -> Option<DawStatus> {
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
            let config_name = normalize_process_name(&config.process_name);
            let additional: Vec<String> = config
                .additional_process_names
                .iter()
                .map(|n| normalize_process_name(n))
                .collect();

            let matches: Vec<_> = self
                .system
                .processes()
                .iter()
                .filter(|(_pid, process)| {
                    let name = normalize_process_name(&process.name().to_string_lossy());
                    name == config_name || additional.iter().any(|prefix| name.starts_with(prefix))
                })
                .collect();

            if matches.is_empty() {
                continue;
            }

            let status = self.aggregate(config, &matches, hide_project_name);
            return Some(status);
        }

        None
    }

    /// Aggregate metrics from all matching processes into a single status
    fn aggregate(
        &self,
        config: &DawConfig,
        matches: &[(&sysinfo::Pid, &sysinfo::Process)],
        hide_project_name: bool,
    ) -> DawStatus {
        let mut total_cpu: f32 = 0.0;
        let mut total_memory: u64 = 0;
        let mut best_title = String::new();
        let mut version = String::new();

        for &(pid, process) in matches {
            let cpu = process.cpu_usage() / self.cpu_count as f32;
            let mem_mb = process.memory() / (1024 * 1024);
            total_cpu += cpu;
            total_memory += mem_mb;

            trace!(
                "  PID {}: \"{}\" — {mem_mb}MB RAM, {cpu:.1}% CPU",
                pid.as_u32(),
                process.name().to_string_lossy(),
            );

            let title = get_window_title(*pid);
            if title.len() > best_title.len() {
                best_title = title;
            }

            if version.is_empty() || version == "0.0.0" {
                let v = get_process_version(process.exe());
                if !v.is_empty() && v != "0.0.0" {
                    version = v;
                }
            }
        }

        if version.is_empty() {
            version = "0.0.0".to_string();
        }

        let project_name = if hide_project_name {
            "(hidden)".to_string()
        } else {
            extract_project_name(&best_title, &config.title_regex)
        };

        debug!(
            "Found {} ({} processes): {}MB RAM, {:.1}% CPU, title=\"{best_title}\"",
            config.display_text,
            matches.len(),
            total_memory,
            total_cpu
        );

        DawStatus {
            is_running: true,
            display_name: config.display_text.clone(),
            project_name,
            cpu_usage: total_cpu,
            memory_mb: total_memory,
            version,
            client_id: config.client_id.clone(),
            hide_version: config.hide_version,
        }
    }
}

/// Ensure a bundled daws.json exists in the config directory.
/// Overwrites the local copy only when the bundled version is newer.
pub(crate) fn ensure_daw_config() -> Result<PathBuf, String> {
    let config_path =
        confy::get_configuration_file_path("dawpresence", None).map_err(|e| e.to_string())?;
    let config_dir = config_path
        .parent()
        .ok_or_else(|| "Couldn't resolve config directory".to_string())?;
    let daws_path = config_dir.join("daws.json");

    let bundled = include_bytes!("../../daws.json");
    let bundled_version = serde_json::from_slice::<DawConfigFile>(bundled).map_or(0, |f| f.version);

    let local_version = std::fs::read_to_string(&daws_path)
        .ok()
        .and_then(|s| serde_json::from_str::<DawConfigFile>(&s).ok())
        .map_or(0, |f| f.version);

    if local_version < bundled_version {
        std::fs::create_dir_all(config_dir)
            .map_err(|e| format!("Couldn't create config directory: {e}"))?;
        std::fs::write(&daws_path, bundled)
            .map_err(|e| format!("Couldn't write daws.json: {e}"))?;
        debug!(
            "Updated daws.json v{local_version} → v{bundled_version} at {}",
            daws_path.display()
        );
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
