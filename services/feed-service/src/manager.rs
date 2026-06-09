use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

type Sender = mpsc::UnboundedSender<Message>;

pub struct ChannelManager {
    channels: Arc<Mutex<HashMap<Uuid, Vec<Sender>>>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        ChannelManager {
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_market_channel(&self, market_id: Uuid) {
        let mut channels = self.channels.lock().await;
        channels.entry(market_id).or_insert_with(Vec::new);
    }

    pub async fn get_market_channel(&self, market_id: Uuid) -> Option<Vec<Sender>> {
        let channels = self.channels.lock().await;
        channels.get(&market_id).cloned()
    }

    pub async fn join_market_channel(&self, market_id: Uuid, sender: Sender) {
        let mut channels = self.channels.lock().await;

        if let Some(senders) = channels.get_mut(&market_id) {
            senders.push(sender);
        }
    }

    pub async fn leave_market_channel(&self, market_id: Uuid, sender: &Sender) {
        let mut channels = self.channels.lock().await;

        if let Some(senders) = channels.get_mut(&market_id) {
            senders.retain(|s| s as *const _ != sender as *const _);
        }
    }

    pub async fn remove_market_channel(&self, market_id: Uuid) {
        let mut channels = self.channels.lock().await;

        channels.remove(&market_id);
    }

    pub async fn broadcast_to_market(&self, market_id: Uuid, message: Message) {
        let mut channels = self.channels.lock().await;

        if let Some(senders) = channels.get_mut(&market_id) {
            for sender in senders {
                sender
                    .send(message.clone())
                    .expect("Failed to send message");
            }
        }
    }
}
