use serde::Deserialize;
use serde::Serialize;

use crate::error::ConfigError;

const DEFAULT_UPDATE_INTERVAL: u64 = 2500;
const MIN_UPDATE_INTERVAL: u64 = 1000;
const MAX_UPDATE_INTERVAL: u64 = 100_000_000;

/// Controls when the Discord presence timer resets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum TimerMode {
    /// Timer resets when DAW opens/closes.
    #[default]
    Session,
    /// Timer resets when project name changes.
    Project,
}

/// User preferences persisted via confy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    #[serde(default)]
    pub(crate) hide_project_name: bool,
    #[serde(default)]
    pub(crate) hide_system_usage: bool,
    #[serde(default = "default_close_to_tray")]
    pub(crate) close_to_tray: bool,
    #[serde(default)]
    pub(crate) timer_mode: TimerMode,
    #[serde(default = "default_update_interval")]
    pub(crate) update_interval: u64,
}

fn default_close_to_tray() -> bool {
    true
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
            timer_mode: TimerMode::Session,
            update_interval: DEFAULT_UPDATE_INTERVAL,
        }
    }
}

impl AppSettings {
    pub(crate) fn load() -> Self {
        confy::load("dawpresence", None).unwrap_or_else(|error| {
            tracing::warn!("Couldn't load settings, using defaults: {error}");
            Self::default()
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_close_to_tray_is_true() {
        let settings = AppSettings::default();
        assert!(settings.close_to_tray);
    }

    #[test]
    fn default_timer_mode_is_session() {
        let settings = AppSettings::default();
        assert_eq!(settings.timer_mode, TimerMode::Session);
    }

    #[test]
    fn close_to_tray_serde_default_is_true() {
        // missing field should default to true, not bool::default() (false)
        let settings: AppSettings = toml::from_str("hide_project_name = false\n").unwrap();
        assert!(settings.close_to_tray);
    }

    #[test]
    fn timer_mode_roundtrip() {
        let settings: AppSettings = toml::from_str("timer_mode = \"Project\"\n").unwrap();
        assert_eq!(settings.timer_mode, TimerMode::Project);
    }

    #[test]
    fn validate_interval_too_low() {
        assert!(AppSettings::validate_update_interval(500).is_err());
    }

    #[test]
    fn validate_interval_valid() {
        assert!(AppSettings::validate_update_interval(2500).is_ok());
    }

    #[test]
    fn set_interval_updates_value() {
        let mut settings = AppSettings::default();
        settings.set_update_interval(5000).unwrap();
        assert_eq!(settings.update_interval, 5000);
    }

    #[test]
    fn set_interval_rejects_invalid() {
        let mut settings = AppSettings::default();
        assert!(settings.set_update_interval(100).is_err());
        assert_eq!(settings.update_interval, 2500);
    }
}
