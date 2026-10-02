use super::*;

#[test]
fn normalize_trims_before_stripping_exe() {
    assert_eq!(normalize_process_name("  REAPER.exe  "), "reaper");
}

#[test]
fn deserialize_defaults_missing_optional_fields() {
    let json = r#"{
        "version": 1,
        "daws": [{
            "ProcessName": "FL64",
            "DisplayText": "FL Studio",
            "TitleRegex": "^(.*?)(?= - FL Studio)",
            "ClientID": "12345"
        }]
    }"#;
    let config: DawConfigFile = serde_json::from_str(json).unwrap();

    assert!(!config.daws[0].hide_version);
    assert_eq!(config.daws[0].additional_process_names, [] as [String; 0]);
}

#[test]
fn primary_process_name_requires_exact_match() {
    let configs = NormalizedConfig::from_configs(vec![DawConfig {
        process_name: "FL".to_owned(),
        display_text: "FL Studio".to_owned(),
        title_regex: String::new(),
        client_id: String::new(),
        hide_version: false,
        additional_process_names: vec![],
    }]);

    assert!(configs[0].matches("fl"));
    assert!(!configs[0].matches("fl64"));
    assert!(!configs[0].matches("flux"));
}

#[test]
fn additional_process_names_match_by_prefix() {
    let configs = NormalizedConfig::from_configs(vec![DawConfig {
        process_name: "BitwigStudioApp".to_owned(),
        display_text: "Bitwig Studio".to_owned(),
        title_regex: String::new(),
        client_id: String::new(),
        hide_version: false,
        additional_process_names: vec!["Bitwig Studio".to_owned(), "BitwigAudioEngine".to_owned()],
    }]);

    assert!(configs[0].matches("bitwigaudioengine-x64-avx2"));
    assert!(configs[0].matches("bitwig studio"));
    assert!(!configs[0].matches("notepad"));
}

#[test]
fn bundled_daws_json_has_required_fields() {
    let content = include_str!("../../../daws.json");
    let config: DawConfigFile = serde_json::from_str(content).unwrap();

    assert!(config.version >= 1);
    assert!(!config.daws.is_empty());

    for daw in &config.daws {
        assert_ne!(daw.process_name, "");
        assert_ne!(daw.display_text, "");
        assert_ne!(daw.title_regex, "");
        assert_ne!(daw.client_id, "");
    }
}
