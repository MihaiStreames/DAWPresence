use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

use crate::logging;

pub struct DiscordManager {
    client: Arc<Mutex<Option<DiscordIpcClient>>>,
    current_client_id: Arc<Mutex<Option<String>>>,
    start_timestamp: Arc<Mutex<Option<i64>>>,
}

impl DiscordManager {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            current_client_id: Arc::new(Mutex::new(None)),
            start_timestamp: Arc::new(Mutex::new(None)),
        }
    }

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
            logging::debug("Discord client ID changed; reconnecting");
            if let Some(ref mut client) = *client_guard {
                let _ = client.clear_activity();
                let _ = client.close();
            }
            *client_guard = None;
            *current_id_guard = None;
            *timestamp_guard = None;
        }

        let mut new_client =
            DiscordIpcClient::new(client_id).map_err(|e| format!("Failed to create client: {e}"))?;

        new_client
            .connect()
            .map_err(|e| format!("Failed to connect: {e}"))?;

        *client_guard = Some(new_client);
        *current_id_guard = Some(client_id.to_string());
        *timestamp_guard = Some(current_timestamp());

        logging::info("Connected to Discord IPC");
        Ok(())
    }

    pub fn update_presence(
        &self,
        details: &str,
        state: &str,
        large_image: &str,
        large_text: &str,
    ) -> Result<(), String> {
        let mut client_guard = self.client.lock().map_err(|e| e.to_string())?;
        let mut current_id_guard = self.current_client_id.lock().map_err(|e| e.to_string())?;
        let mut timestamp_guard = self.start_timestamp.lock().map_err(|e| e.to_string())?;

        let Some(ref mut client) = *client_guard else {
            return Ok(());
        };

        let timestamp = timestamp_guard.unwrap_or_else(current_timestamp);

        let build_activity = || {
            activity::Activity::new()
                .details(details)
                .state(state)
                .assets(
                    activity::Assets::new()
                        .large_image(large_image)
                        .large_text(large_text),
                )
                .timestamps(activity::Timestamps::new().start(timestamp))
        };

        if let Err(error) = client.set_activity(build_activity()) {
            logging::warn(format!("Discord set_activity failed: {error}"));
            if let Err(reconnect_error) = client.reconnect() {
                logging::error(format!("Discord reconnect failed: {reconnect_error}"));
                let _ = client.close();
                *client_guard = None;
                *current_id_guard = None;
                *timestamp_guard = None;
                return Err(format!(
                    "Failed to set activity: {error}; reconnect also failed: {reconnect_error}"
                ));
            }

            client
                .set_activity(build_activity())
                .map_err(|e| format!("Failed to set activity after reconnect: {e}"))?;

            logging::info("Discord reconnected");
        }

        Ok(())
    }

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

        logging::debug("Disconnected from Discord IPC");
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
