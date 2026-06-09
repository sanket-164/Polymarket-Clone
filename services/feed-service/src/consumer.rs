use crate::manager::ChannelManager;
use common::{constant::FEED_STREAM, model::FeedMessage, nats_handler::NatsHandler};
use futures::StreamExt;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;

pub async fn start_consumer(nats_handler: Arc<NatsHandler>, channel_manager: Arc<ChannelManager>) {
    let mut message_stream = match nats_handler.get_message_stream(FEED_STREAM).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to get NATS message stream: {}", e);
            return;
        }
    };

    while let Some(msg) = message_stream.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                eprintln!("NATS message error: {}", e);
                continue;
            }
        };

        let feed_message: FeedMessage = match serde_json::from_slice(&msg.payload) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to deserialize NATS message: {}", e);
                let _ = msg.ack().await;
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

        if let Err(e) = msg.ack().await {
            eprintln!("NATS ack failed: {}", e);
        }
    }
}
