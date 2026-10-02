use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use tracing::debug;

use crate::error::ConfigError;

/// Versioned wrapper for `daws.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DawConfigFile {
    version: u32,
    daws: Vec<DawConfig>,
}

/// DAW configuration from `daws.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DawConfig {
    #[serde(rename = "ProcessName")]
    process_name: String,
    #[serde(rename = "DisplayText")]
    display_text: String,
    #[serde(rename = "TitleRegex")]
    title_regex: String,
    #[serde(rename = "ClientID")]
    client_id: String,
    #[serde(rename = "HideVersion")]
    #[serde(default)]
    hide_version: bool,
    #[serde(rename = "AdditionalProcessNames")]
    #[serde(default)]
    additional_process_names: Vec<String>,
}

/// Load DAW configs from a JSON file
pub(crate) fn load_configs(path: &Path) -> Result<Vec<DawConfig>, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let file: DawConfigFile = serde_json::from_str(&content)?;
    Ok(file.daws)
}

/// Ensure a bundled `daws.json` exists in the config directory or
/// overwrite the local copy only when the bundled version is newer
pub(crate) fn ensure_daw_config() -> Result<PathBuf, ConfigError> {
    let config_path =
        confy::get_configuration_file_path("dawpresence", None).map_err(|e| ConfigError::InitFailed(e.to_string()))?;
    let config_dir = config_path.parent().ok_or(ConfigError::NoConfigDir)?;
    let daws_path = config_dir.join("daws.json");

    let bundled = include_bytes!("../../daws.json");
    let bundled_version = serde_json::from_slice::<DawConfigFile>(bundled).map_or(0, |f| f.version);

    let local_version = std::fs::read_to_string(&daws_path)
        .ok()
        .and_then(|s| serde_json::from_str::<DawConfigFile>(&s).ok())
        .map_or(0, |f| f.version);

    if local_version < bundled_version {
        std::fs::create_dir_all(config_dir)?;
        std::fs::write(&daws_path, bundled)?;

        debug!(
            "Updated daws.json v{local_version} -> v{bundled_version} at {}",
            daws_path.display()
        );
    }

    Ok(daws_path)
}

/// Pre-normalized DAW config (for fast matching during scanning)
pub(super) struct NormalizedConfig {
    config: DawConfig,
    normalized_name: String,
    additional_prefixes: Vec<String>,
}

impl NormalizedConfig {
    pub(super) fn from_configs(configs: Vec<DawConfig>) -> Vec<Self> {
        configs
            .into_iter()
            .map(|config| {
                let normalized_name = normalize_process_name(&config.process_name);

                let additional_prefixes = config
                    .additional_process_names
                    .iter()
                    .map(|n| normalize_process_name(n))
                    .collect();

                Self {
                    config,
                    normalized_name,
                    additional_prefixes,
                }
            })
            .collect()
    }

    /// Check if a normalized process name matches this config
    pub(super) fn matches(&self, process_name: &str) -> bool {
        // exact match instead of starts_with (#46)
        process_name == self.normalized_name
            || self
                .additional_prefixes
                .iter()
                .any(|prefix| process_name.starts_with(prefix))
    }

    pub(super) fn display_text(&self) -> &str {
        &self.config.display_text
    }

    pub(super) fn title_regex(&self) -> &str {
        &self.config.title_regex
    }

    pub(super) fn client_id(&self) -> &str {
        &self.config.client_id
    }

    pub(super) const fn hide_version(&self) -> bool {
        self.config.hide_version
    }
}

/// Normalize a process name for comparison (lowercase, strip .exe)
pub(super) fn normalize_process_name(name: &str) -> String {
    let lower = name.trim().to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_owned()
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
