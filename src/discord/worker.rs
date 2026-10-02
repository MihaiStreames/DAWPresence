use std::sync::mpsc;

use iced::Subscription;
use iced::futures::channel::mpsc::Sender;
use iced::futures::future;
use tracing::debug;
use tracing::warn;

use super::DiscordManager;
use super::PresenceRequest;
use crate::state::Message;

/// Handle for pushing presence requests to the Discord worker thread
///
/// `None` means no DAW is running, so the worker disconnects
#[derive(Debug, Clone)]
pub(crate) struct DiscordSender(mpsc::Sender<Option<PresenceRequest>>);

impl DiscordSender {
    /// Queue a presence request; returns `false` once the worker has exited
    pub(crate) fn send(&self, request: Option<PresenceRequest>) -> bool {
        if self.0.send(request).is_err() {
            warn!("Discord worker stopped, presence updates disabled");
            return false;
        }

        true
    }
}

/// Spawn the Discord worker thread and hand its sender to the app
pub(crate) fn discord_subscription() -> Subscription<Message> {
    Subscription::run(|| {
        iced::stream::channel(100, |mut output: Sender<Message>| async move {
            let (sender, receiver) = mpsc::channel();
            let worker_output = output.clone();

            std::thread::Builder::new()
                .name("discord-worker".into())
                .spawn(move || run_worker(&receiver, worker_output))
                .expect("Couldn't spawn Discord worker thread");

            if output.try_send(Message::DiscordReady(DiscordSender(sender))).is_err() {
                warn!("App channel closed before Discord worker was ready");
            }

            future::pending::<()>().await;
        })
    })
}

fn run_worker(receiver: &mpsc::Receiver<Option<PresenceRequest>>, mut output: Sender<Message>) {
    let mut discord = DiscordManager::new();
    let mut was_connected = false;

    while let Ok(request) = receiver.recv() {
        // a hung discord shouldn't replay a backlog of stale ticks once it recovers
        let request = receiver.try_iter().last().unwrap_or(request);

        if let Err(error) = discord.apply_request(request.as_ref()) {
            warn!("Couldn't update Discord presence: {error}");
        }

        let is_connected = discord.is_connected();
        if is_connected == was_connected {
            continue;
        }

        match output.try_send(Message::DiscordConnected(is_connected)) {
            Ok(()) => was_connected = is_connected,

            Err(error) if error.is_disconnected() => {
                warn!("App channel closed, stopping Discord worker");
                break;
            }

            // leave was_connected stale so the next request retries
            Err(_) => warn!("App channel full, retrying Discord connection state next update"),
        }
    }

    discord.disconnect();
    debug!("Discord worker exited");
}
