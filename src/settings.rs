//! User settings with confy persistence.

use serde::Deserialize;
use serde::Serialize;

use crate::error::ConfigError;

const DEFAULT_UPDATE_INTERVAL: u64 = 2500;
const MIN_UPDATE_INTERVAL: u64 = 1000;
const MAX_UPDATE_INTERVAL: u64 = 100_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// User preferences persisted via confy.
pub(crate) struct AppSettings {
    #[serde(default)]
    pub(crate) hide_project_name: bool,
    #[serde(default)]
    pub(crate) hide_system_usage: bool,
    #[serde(default)]
    pub(crate) close_to_tray: bool,
    #[serde(default = "default_update_interval")]
    pub(crate) update_interval: u64,
}

fn default_update_interval() -> u64 {
    DEFAULT_UPDATE_INTERVAL
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hide_project_name: false,
            hide_system_usage: false,
            close_to_tray: true,
            update_interval: DEFAULT_UPDATE_INTERVAL,
        }
    }
}

impl AppSettings {
    pub(crate) fn load() -> Self {
        confy::load("dawpresence", None).unwrap_or_default()
    }

    pub(crate) fn save(&self) -> Result<(), ConfigError> {
        confy::store("dawpresence", None, self).map_err(|e| ConfigError::SaveFailed(e.to_string()))
    }

    pub(crate) fn set_update_interval(&mut self, interval: u64) -> Result<(), ConfigError> {
        Self::validate_update_interval(interval)?;
        self.update_interval = interval;
        Ok(())
    }

    pub(crate) fn validate_update_interval(interval: u64) -> Result<(), ConfigError> {
        if !(MIN_UPDATE_INTERVAL..=MAX_UPDATE_INTERVAL).contains(&interval) {
            return Err(ConfigError::InvalidInterval {
                min: MIN_UPDATE_INTERVAL,
                max: MAX_UPDATE_INTERVAL,
            });
        }

        Ok(())
    }
}
