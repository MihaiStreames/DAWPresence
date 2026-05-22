mod presence;

use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use discord_rich_presence::DiscordIpc;
use discord_rich_presence::DiscordIpcClient;
use discord_rich_presence::activity;
use presence::DiscordPresence;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::trace;
use tracing::warn;

use crate::daw::DawStatus;
use crate::error::DiscordError;
use crate::settings::AppSettings;

struct DiscordState {
    client: Option<DiscordIpcClient>,
    client_id: Option<String>,
    start_timestamp: Option<i64>,
}

impl DiscordState {
    fn clear(&mut self) {
        self.client = None;
        self.client_id = None;
        self.start_timestamp = None;
    }
}

/// Manages Discord IPC connection, reconnection, and presence updates.
pub(crate) struct DiscordManager {
    state: Mutex<DiscordState>,
}

impl DiscordManager {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(DiscordState {
                client: None,
                client_id: None,
                start_timestamp: None,
            }),
        }
    }

    pub(crate) fn is_connected(&self) -> bool {
        self.lock().client.is_some()
    }

    /// Connect (or reconnect if client ID changed) to Discord IPC.
    pub(crate) fn connect(&self, client_id: &str) -> Result<(), DiscordError> {
        let mut s = self.lock();

        if s.client_id
            .as_ref()
            .is_some_and(|id| id == client_id && s.client.is_some())
        {
            return Ok(());
        }

        if s.client_id.is_some() {
            debug!("Client ID changed, reconnecting...");

            if let Some(ref mut client) = s.client {
                let _ = client.clear_activity();
                let _ = client.close();
            }

            s.clear();
        }

        let mut new_client = DiscordIpcClient::new(client_id);
        new_client
            .connect()
            .map_err(|e| DiscordError::Connect(e.to_string()))?;

        s.client = Some(new_client);
        s.client_id = Some(client_id.to_string());
        s.start_timestamp = Some(current_timestamp());

        info!("Connected to Discord RPC");

        Ok(())
    }

    /// Push presence to Discord, retrying once on connection failure.
    fn update_presence(&self, presence: &DiscordPresence) -> Result<(), DiscordError> {
        let mut s = self.lock();

        let timestamp = s.start_timestamp.unwrap_or_else(current_timestamp);

        let Some(ref mut client) = s.client else {
            return Ok(());
        };

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
            warn!("Couldn't set activity: {e}, trying to reconnect...");

            if let Err(reconnect_err) = client.reconnect() {
                error!("Couldn't reconnect: {reconnect_err}");

                let _ = client.close();

                s.clear();

                return Err(DiscordError::Reconnect {
                    activity: e.to_string(),
                    reconnect: reconnect_err.to_string(),
                });
            }

            client
                .set_activity(build_activity())
                .map_err(|e| DiscordError::Activity(e.to_string()))?;

            info!("Reconnected to Discord RPC");
        }

        Ok(())
    }

    /// Reset the presence timer to now (for project-based timer mode).
    pub(crate) fn reset_timestamp(&self) {
        let mut s = self.lock();
        s.start_timestamp = Some(current_timestamp());
    }

    pub(crate) fn disconnect(&self) {
        let mut s = self.lock();

        if let Some(ref mut client) = s.client {
            let _ = client.clear_activity();
            let _ = client.close();
        }

        s.clear();

        debug!("Disconnected from Discord RPC");
    }

    /// Update presence from DAW status, or disconnect if no DAW running.
    pub(crate) fn update_from_daw_status(
        &self,
        daw_status: Option<&DawStatus>,
        settings: &AppSettings,
    ) -> Result<(), DiscordError> {
        let Some(status) = daw_status else {
            if self.is_connected() {
                self.disconnect();
            }
            return Ok(());
        };

        self.connect(&status.client_id)?;

        let presence = DiscordPresence::from_daw_status(status, settings);
        self.update_presence(&presence)?;

        trace!("Presence updated: {}", presence.details);

        Ok(())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, DiscordState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
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
        .unwrap_or(Duration::ZERO)
        .as_secs()
        .cast_signed()
}
