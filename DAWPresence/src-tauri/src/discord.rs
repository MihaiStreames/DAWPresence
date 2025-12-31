use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

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

        if let Some(ref current_id) = *current_id_guard {
            if current_id == client_id {
                return Ok(());
            }
            if let Some(ref mut client) = *client_guard {
                let _ = client.close();
            }
        }

        let mut new_client =
            DiscordIpcClient::new(client_id).map_err(|e| format!("Failed to create client: {e}"))?;

        new_client
            .connect()
            .map_err(|e| format!("Failed to connect: {e}"))?;

        *client_guard = Some(new_client);
        *current_id_guard = Some(client_id.to_string());
        *timestamp_guard = Some(current_timestamp());

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
        let timestamp_guard = self.start_timestamp.lock().map_err(|e| e.to_string())?;

        let Some(ref mut client) = *client_guard else {
            return Ok(());
        };

        let timestamp = timestamp_guard.unwrap_or_else(current_timestamp);

        let activity = activity::Activity::new()
            .details(details)
            .state(state)
            .assets(
                activity::Assets::new()
                    .large_image(large_image)
                    .large_text(large_text),
            )
            .timestamps(activity::Timestamps::new().start(timestamp));

        client
            .set_activity(activity)
            .map_err(|e| format!("Failed to set activity: {e}"))
    }

    pub fn disconnect(&self) -> Result<(), String> {
        let mut client_guard = self.client.lock().map_err(|e| e.to_string())?;
        let mut current_id_guard = self.current_client_id.lock().map_err(|e| e.to_string())?;
        let mut timestamp_guard = self.start_timestamp.lock().map_err(|e| e.to_string())?;

        if let Some(ref mut client) = *client_guard {
            let _ = client.close();
        }

        *client_guard = None;
        *current_id_guard = None;
        *timestamp_guard = None;

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
