use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use tracing::{debug, error, info, warn};

use crate::daw::DawStatus;
use crate::settings::AppSettings;
use crate::version::APP_VERSION;

/// Rich Presence data to display on Discord
pub struct DiscordPresence {
    pub details: String,
    pub state: String,
    pub large_image: String,
    pub large_text: String,
}

impl DiscordPresence {
    /// Build presence from current DAW status
    pub fn from_daw_status(daw_status: &DawStatus, settings: &AppSettings) -> Self {
        let project = if settings.hide_project_name {
            "(hidden)".to_string()
        } else {
            daw_status.project_name.clone()
        };

        let details = if project == "None" || project == "Untitled" {
            "Opening an untitled project".to_string()
        } else {
            format!("Opening project: {project}")
        };

        let state = if settings.hide_system_usage {
            format!("Using {}", daw_status.display_name)
        } else {
            let mut parts = Vec::new();
            if !daw_status.hide_version && daw_status.version != "0.0.0" {
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

/// Manages Discord IPC connection and presence updates
pub struct DiscordManager {
    client: Mutex<Option<DiscordIpcClient>>,
    current_client_id: Mutex<Option<String>>,
    start_timestamp: Mutex<Option<i64>>,
}

impl DiscordManager {
    /// Create a new Discord manager (not connected yet)
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            current_client_id: Mutex::new(None),
            start_timestamp: Mutex::new(None),
        }
    }

    /// Check if currently connected to Discord
    pub fn is_connected(&self) -> bool {
        self.client.lock().unwrap().is_some()
    }

    /// Connect to Discord with the given client ID (reconnects if ID changed)
    pub fn connect(&self, client_id: &str) -> Result<(), String> {
        let mut client_guard = self.client.lock().map_err(|e| e.to_string())?;
        let mut current_id_guard = self.current_client_id.lock().map_err(|e| e.to_string())?;
        let mut timestamp_guard = self.start_timestamp.lock().map_err(|e| e.to_string())?;

        if current_id_guard
            .as_ref()
            .is_some_and(|current_id| current_id == client_id && client_guard.is_some())
        {
            return Ok(());
        }

        if current_id_guard.is_some() {
            debug!("Client ID changed, reconnecting...");
            if let Some(ref mut client) = *client_guard {
                let _ = client.clear_activity();
                let _ = client.close();
            }
            *client_guard = None;
            *current_id_guard = None;
            *timestamp_guard = None;
        }

        let mut new_client = DiscordIpcClient::new(client_id);

        new_client
            .connect()
            .map_err(|e| format!("Couldn't connect: {e}"))?;

        *client_guard = Some(new_client);
        *current_id_guard = Some(client_id.to_string());
        *timestamp_guard = Some(current_timestamp());

        info!("Connected to Discord RPC");
        Ok(())
    }

    /// Update the current presence (auto-reconnects on failure)
    pub fn update_presence(&self, presence: &DiscordPresence) -> Result<(), String> {
        let mut client_guard = self.client.lock().map_err(|e| e.to_string())?;
        let mut current_id_guard = self.current_client_id.lock().map_err(|e| e.to_string())?;
        let mut timestamp_guard = self.start_timestamp.lock().map_err(|e| e.to_string())?;

        let Some(ref mut client) = *client_guard else {
            return Ok(());
        };

        let timestamp = timestamp_guard.unwrap_or_else(current_timestamp);

        let build_activity = || {
            activity::Activity::new()
                .details(&presence.details)
                .state(&presence.state)
                .assets(
                    activity::Assets::new()
                        .large_image(&presence.large_image)
                        .large_text(&presence.large_text),
                )
                .timestamps(activity::Timestamps::new().start(timestamp))
        };

        if let Err(e) = client.set_activity(build_activity()) {
            warn!("set_activity failed: {e}, trying to reconnect...");
            if let Err(reconnect_err) = client.reconnect() {
                error!("Reconnect failed: {reconnect_err}");
                let _ = client.close();
                *client_guard = None;
                *current_id_guard = None;
                *timestamp_guard = None;
                return Err(format!(
                    "set_activity failed: {e}; reconnect also failed: {reconnect_err}"
                ));
            }

            client
                .set_activity(build_activity())
                .map_err(|e| format!("set_activity failed after reconnect: {e}"))?;

            info!("Reconnected to Discord RPC");
        }

        Ok(())
    }

    /// Disconnect from Discord and clear presence
    pub fn disconnect(&self) -> Result<(), String> {
        let mut client_guard = self.client.lock().map_err(|e| e.to_string())?;
        let mut current_id_guard = self.current_client_id.lock().map_err(|e| e.to_string())?;
        let mut timestamp_guard = self.start_timestamp.lock().map_err(|e| e.to_string())?;

        if let Some(ref mut client) = *client_guard {
            let _ = client.clear_activity();
            let _ = client.close();
        }

        *client_guard = None;
        *current_id_guard = None;
        *timestamp_guard = None;

        debug!("Disconnected from Discord RPC");
        Ok(())
    }

    /// Convenience method: update presence from DAW status, or disconnect if no DAW
    pub fn update_from_daw_status(
        &self,
        daw_status: Option<&DawStatus>,
        settings: &AppSettings,
    ) -> Result<(), String> {
        let Some(status) = daw_status else {
            self.disconnect()?;
            return Ok(());
        };

        self.connect(&status.client_id)?;
        let presence = DiscordPresence::from_daw_status(status, settings);
        self.update_presence(&presence)?;
        debug!("Presence updated: {}", presence.details);
        Ok(())
    }
}

impl Default for DiscordManager {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
