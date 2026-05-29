use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use tracing::debug;

use crate::error::ConfigError;

/// Versioned wrapper for `daws.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DawConfigFile {
    version: u32,
    daws: Vec<DawConfig>,
}

/// DAW configuration from `daws.json`.
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

/// Load DAW configs from a JSON file.
pub(crate) fn load_configs(path: &Path) -> Result<Vec<DawConfig>, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let file: DawConfigFile = serde_json::from_str(&content)?;
    Ok(file.daws)
}

/// Ensure a bundled `daws.json` exists in the config directory.
///
/// Overwrites the local copy only when the bundled version is newer.
pub(crate) fn ensure_daw_config() -> Result<PathBuf, ConfigError> {
    let config_path = confy::get_configuration_file_path("dawpresence", None)
        .map_err(|e| ConfigError::InitFailed(e.to_string()))?;
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

/// Pre-normalized DAW config (for fast matching during scanning).
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

    /// Check if a normalized process name matches this config.
    pub(super) fn matches(&self, process_name: &str) -> bool {
        process_name.starts_with(&self.normalized_name)
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

/// Normalize a process name for comparison (lowercase, strip .exe).
pub(super) fn normalize_process_name(name: &str) -> String {
    let lower = name.trim().to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_exe() {
        assert_eq!(normalize_process_name("FL64.exe"), "fl64");
    }

    #[test]
    fn normalize_lowercases() {
        assert_eq!(normalize_process_name("BitwigStudioApp"), "bitwigstudioapp");
    }

    #[test]
    fn normalize_trims_whitespace() {
        assert_eq!(normalize_process_name("  reaper.exe  "), "reaper");
    }

    #[test]
    fn normalize_no_exe() {
        assert_eq!(normalize_process_name("lmms"), "lmms");
    }

    #[test]
    fn deserialize_daw_config() {
        let json = r#"{
            "version": 1,
            "daws": [{
                "ProcessName": "FL64",
                "DisplayText": "FL Studio",
                "TitleRegex": "^(.*?)(?= - FL Studio)",
                "ClientID": "12345",
                "HideVersion": true
            }]
        }"#;
        let config: DawConfigFile = serde_json::from_str(json).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.daws.len(), 1);
        assert_eq!(config.daws[0].process_name, "FL64");
        assert!(config.daws[0].hide_version);
        assert!(config.daws[0].additional_process_names.is_empty());
    }

    #[test]
    fn deserialize_with_additional_processes() {
        let json = r#"{
            "version": 1,
            "daws": [{
                "ProcessName": "BitwigStudioApp",
                "DisplayText": "Bitwig Studio",
                "TitleRegex": "(?<=Bitwig Studio - ).*",
                "ClientID": "12345",
                "HideVersion": false,
                "AdditionalProcessNames": ["Bitwig Studio", "BitwigAudioEngine"]
            }]
        }"#;
        let config: DawConfigFile = serde_json::from_str(json).unwrap();
        assert_eq!(config.daws[0].additional_process_names.len(), 2);
    }

    #[test]
    fn match_fl_studio_prefix() {
        let configs = NormalizedConfig::from_configs(vec![DawConfig {
            process_name: "FL".to_owned(),
            display_text: "FL Studio".to_owned(),
            title_regex: String::new(),
            client_id: String::new(),
            hide_version: false,
            additional_process_names: vec![],
        }]);
        assert!(configs[0].matches("fl"));
        assert!(configs[0].matches("fl64"));
        assert!(!configs[0].matches("firefox"));
    }

    #[test]
    fn match_bitwig_additional_prefixes() {
        let configs = NormalizedConfig::from_configs(vec![DawConfig {
            process_name: "BitwigStudioApp".to_owned(),
            display_text: "Bitwig Studio".to_owned(),
            title_regex: String::new(),
            client_id: String::new(),
            hide_version: false,
            additional_process_names: vec![
                "Bitwig Studio".to_owned(),
                "BitwigAudioEngine".to_owned(),
            ],
        }]);
        assert!(configs[0].matches("bitwigstudioapp"));
        assert!(configs[0].matches("bitwigaudioengine-x64-avx2"));
        assert!(configs[0].matches("bitwig studio"));
        assert!(!configs[0].matches("notepad"));
    }

    #[test]
    fn deserialize_bundled_daws_json() {
        let content = include_str!("../../daws.json");
        let config: DawConfigFile = serde_json::from_str(content).unwrap();
        assert!(config.version >= 1);
        assert!(!config.daws.is_empty());
        for daw in &config.daws {
            assert!(!daw.process_name.is_empty());
            assert!(!daw.display_text.is_empty());
            assert!(!daw.title_regex.is_empty());
            assert!(!daw.client_id.is_empty());
        }
    }
}
