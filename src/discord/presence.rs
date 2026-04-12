//! Discord Rich Presence data formatting.

use crate::daw::DawStatus;
use crate::daw::UNKNOWN_PROJECT;
use crate::daw::UNKNOWN_VERSION;
use crate::daw::UNTITLED_PROJECT;
use crate::settings::AppSettings;
use crate::version::APP_VERSION;

/// Rich Presence data to display on Discord.
pub(super) struct DiscordPresence {
    pub(super) details: String,
    pub(super) state: String,
    pub(super) large_image: String,
    pub(super) large_text: String,
}

impl DiscordPresence {
    /// Build presence from current DAW status.
    pub(super) fn from_daw_status(daw_status: &DawStatus, settings: &AppSettings) -> Self {
        let project = if settings.hide_project_name {
            "(hidden)".to_string()
        } else {
            daw_status.project_name.clone()
        };

        let details = if project == UNKNOWN_PROJECT || project == UNTITLED_PROJECT {
            "Opening an untitled project".to_string()
        } else {
            format!("Opening project: {project}")
        };

        let state = if settings.hide_system_usage {
            format!("Using {}", daw_status.display_name)
        } else {
            let mut parts = Vec::new();

            if !daw_status.hide_version && daw_status.version != UNKNOWN_VERSION {
                parts.push(format!("v{}", daw_status.version));
            }

            parts.push(format!("{} CPU", daw_status.cpu_usage_str()));
            parts.push(format!("{} RAM", daw_status.ram_usage_str()));
            parts.join(", ")
        };

        Self {
            details,
            state,
            large_image: "icon".to_string(),
            large_text: format!("DAWPresence v{APP_VERSION}"),
        }
    }
}
