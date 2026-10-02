mod presence;
mod worker;

use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use discord_rich_presence::DiscordIpc as _;
use discord_rich_presence::DiscordIpcClient;
use discord_rich_presence::activity;
use presence::DiscordPresence;
pub(crate) use presence::PresenceRequest;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::trace;
use tracing::warn;
pub(crate) use worker::DiscordSender;
pub(crate) use worker::discord_subscription;

use crate::error::DiscordError;

/// Manages Discord IPC connection, reconnection, and presence updates
///
/// Owned solely by the Discord worker thread, so IPC never blocks the UI
struct DiscordManager {
    client: Option<DiscordIpcClient>,
    client_id: Option<String>,
    start_timestamp: Option<i64>,
}

impl DiscordManager {
    const fn new() -> Self {
        Self {
            client: None,
            client_id: None,
            start_timestamp: None,
        }
    }

    const fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    /// Connect (or reconnect if client ID changed) to Discord IPC
    fn connect(&mut self, client_id: &str) -> Result<(), DiscordError> {
        if self.client.is_some() && self.client_id.as_deref() == Some(client_id) {
            return Ok(());
        }

        if self.client_id.is_some() {
            debug!("Client ID changed, reconnecting...");

            if let Some(ref mut client) = self.client {
                let _ = client.clear_activity();
                let _ = client.close();
            }

            self.clear();
        }

        let mut new_client = DiscordIpcClient::new(client_id);
        new_client.connect().map_err(|e| DiscordError::Connect(e.to_string()))?;

        self.client = Some(new_client);
        self.client_id = Some(client_id.to_owned());
        self.start_timestamp = Some(current_timestamp());

        info!("Connected to Discord RPC");

        Ok(())
    }

    fn update_presence(&mut self, presence: &DiscordPresence) -> Result<(), DiscordError> {
        let timestamp = self.start_timestamp.unwrap_or_else(current_timestamp);

        let Some(ref mut client) = self.client else {
            debug_assert!(false, "update_presence called with no client");
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

        // retry once on connection failure
        if let Err(e) = client.set_activity(build_activity()) {
            warn!("Couldn't set activity: {e}, trying to reconnect...");

            if let Err(reconnect_err) = client.reconnect() {
                error!("Couldn't reconnect: {reconnect_err}");

                let _ = client.close();

                self.clear();

                return Err(DiscordError::Reconnect {
                    activity: e.to_string(),
                    reconnect: reconnect_err.to_string(),
                });
            }

            // push presence
            client
                .set_activity(build_activity())
                .map_err(|e| DiscordError::Activity(e.to_string()))?;

            info!("Reconnected to Discord RPC");
        }

        Ok(())
    }

    fn disconnect(&mut self) {
        if let Some(ref mut client) = self.client {
            let _ = client.clear_activity();
            let _ = client.close();
        }

        self.clear();

        debug!("Disconnected from Discord RPC");
    }

    /// Push the requested presence, or disconnect when no DAW is running (`None`)
    fn apply_request(&mut self, request: Option<&PresenceRequest>) -> Result<(), DiscordError> {
        let Some(request) = request else {
            if self.is_connected() {
                self.disconnect();
            }
            return Ok(());
        };

        self.connect(&request.client_id)?;
        self.update_presence(&request.presence)?;

        trace!("Presence updated: {}", request.presence.details);

        Ok(())
    }

    fn clear(&mut self) {
        self.client = None;
        self.client_id = None;
        self.start_timestamp = None;
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        .cast_signed()
}
