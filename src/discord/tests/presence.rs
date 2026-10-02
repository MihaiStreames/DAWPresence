use super::*;

fn running_status() -> DawStatus {
    DawStatus {
        is_running: true,
        display_name: "FL Studio".to_owned(),
        project_name: "song".to_owned(),
        cpu_usage: 5.0,
        memory_mb: 512,
        version: "21.0.0".to_owned(),
        client_id: "1234".to_owned(),
        hide_version: false,
    }
}

#[test]
fn request_hides_project_name_when_enabled() {
    let settings = AppSettings {
        hide_project_name: true,
        ..AppSettings::default()
    };

    let request = PresenceRequest::from_daw_status(&running_status(), &settings);
    assert_eq!(request.presence.details, "Opening project: (hidden)");
}

#[test]
fn request_hides_system_usage_when_enabled() {
    let settings = AppSettings {
        hide_system_usage: true,
        ..AppSettings::default()
    };

    let request = PresenceRequest::from_daw_status(&running_status(), &settings);
    assert_eq!(request.presence.state, "Using FL Studio");
}

#[test]
fn request_omits_unknown_version() {
    let status = DawStatus {
        version: UNKNOWN_VERSION.to_owned(),
        ..running_status()
    };

    let request = PresenceRequest::from_daw_status(&status, &AppSettings::default());
    assert_eq!(request.presence.state, "5.00% CPU, 512MB RAM");
}

#[test]
fn untitled_project_shows_generic_details() {
    let status = DawStatus {
        project_name: UNTITLED_PROJECT.to_owned(),
        ..running_status()
    };

    let request = PresenceRequest::from_daw_status(&status, &AppSettings::default());
    assert_eq!(request.presence.details, "Opening an untitled project");
}
