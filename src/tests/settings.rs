use super::*;

#[test]
fn close_to_tray_serde_default_is_true() {
    // missing field should default to true, not bool::default() (false)
    let settings: AppSettings = toml::from_str("hide_project_name = false\n").unwrap();
    assert!(settings.close_to_tray);
}

#[test]
fn was_hidden_serde_default_is_true() {
    // existing configs keep starting in tray after upgrading
    let settings: AppSettings = toml::from_str("hide_project_name = false\n").unwrap();
    assert!(settings.was_hidden);
}

#[test]
fn manual_launch_always_shows_window() {
    let settings = AppSettings::default();
    assert!(!settings.should_start_hidden(false));
}

#[test]
fn autostart_shows_window_when_left_open() {
    let settings = AppSettings {
        was_hidden: false,
        ..AppSettings::default()
    };
    assert!(!settings.should_start_hidden(true));
}

#[test]
fn validate_interval_accepts_bounds() {
    AppSettings::validate_update_interval(MIN_UPDATE_INTERVAL).unwrap();
    AppSettings::validate_update_interval(MAX_UPDATE_INTERVAL).unwrap();
}

#[test]
fn validate_interval_rejects_out_of_bounds() {
    assert!(AppSettings::validate_update_interval(MIN_UPDATE_INTERVAL - 1).is_err());
    assert!(AppSettings::validate_update_interval(MAX_UPDATE_INTERVAL + 1).is_err());
}

#[test]
fn set_interval_keeps_previous_value_on_error() {
    let mut settings = AppSettings::default();
    assert!(settings.set_update_interval(100).is_err());
    assert_eq!(settings.update_interval, DEFAULT_UPDATE_INTERVAL);
}
