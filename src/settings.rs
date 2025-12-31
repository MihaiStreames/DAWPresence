use serde::{Deserialize, Serialize};

const DEFAULT_UPDATE_INTERVAL: u64 = 2500;
const MIN_UPDATE_INTERVAL: u64 = 1000;
const MAX_UPDATE_INTERVAL: u64 = 100_000_000;

/// User-configurable app settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub hide_project_name: bool,
    #[serde(default)]
    pub hide_system_usage: bool,
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,
}

fn default_update_interval() -> u64 {
    DEFAULT_UPDATE_INTERVAL
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hide_project_name: false,
            hide_system_usage: false,
            update_interval: DEFAULT_UPDATE_INTERVAL,
        }
    }
}

impl AppSettings {
    /// Load settings from disk, or return defaults if not found
    pub fn load() -> Self {
        confy::load("dawpresence", None).unwrap_or_default()
    }

    /// Save settings to disk
    pub fn save(&self) -> Result<(), String> {
        confy::store("dawpresence", None, self).map_err(|e| e.to_string())
    }

    /// Set update interval with validation (1000ms - 100,000,000ms)
    pub fn set_update_interval(&mut self, interval: u64) -> Result<(), String> {
        if !(MIN_UPDATE_INTERVAL..=MAX_UPDATE_INTERVAL).contains(&interval) {
            return Err(format!(
                "Interval must be between {}ms and {}ms",
                MIN_UPDATE_INTERVAL, MAX_UPDATE_INTERVAL
            ));
        }
        self.update_interval = interval;
        Ok(())
    }

    /// Toggle project name visibility in presence
    pub fn toggle_hide_project_name(&mut self) {
        self.hide_project_name = !self.hide_project_name;
    }

    /// Toggle system usage (CPU/RAM) visibility in presence
    pub fn toggle_hide_system_usage(&mut self) {
        self.hide_system_usage = !self.hide_system_usage;
    }
}
