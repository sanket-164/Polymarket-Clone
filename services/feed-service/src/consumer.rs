use crate::{manager::ChannelManager, nats_handler::NatsClient};
use common::model::FeedMessage;
use futures::StreamExt;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;

pub async fn start_consumer(nats_client: Arc<NatsClient>, channel_manager: Arc<ChannelManager>) {
    let mut subscriber = match nats_client.get_feed_subscriber().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to get NATS message stream: {}", e);
            return;
        }
    };

    while let Some(msg) = subscriber.next().await {
        let feed_message: FeedMessage = match serde_json::from_slice(&msg.payload) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to deserialize NATS message: {}", e);
                continue;
            }
        };

        match feed_message {
            FeedMessage::CreateMarket { market_id } => {
                channel_manager.create_market_channel(market_id).await;
                println!("Created channel for market {}", market_id);
            }

            FeedMessage::RemoveMarket { market_id } => {
                channel_manager.remove_market_channel(market_id).await;
                println!("Removed channel for market {}", market_id);
            }

            FeedMessage::OrderFeed { feed } => {
                // Broadcast the new order state to all subscribers of this market
                match serde_json::to_string(&feed) {
                    Ok(text) => {
                        channel_manager
                            .broadcast_to_market(feed.market_id, Message::Text(text.into()))
                            .await;
                    }
                    Err(e) => eprintln!("Failed to serialize OrderPlaced message: {}", e),
                }
            }
        }
    }
}
