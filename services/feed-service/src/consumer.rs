use crate::manager::ChannelManager;
use common::{
    constant::{FEED_CREATE_MARKET, FEED_MARKET_ORDER, FEED_REMOVE_MARKET, FEED_STREAM},
    model::{FeedMessage, ServerMessage},
    nats_handler::NatsHandler,
};
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

        let matcher_message: FeedMessage = match serde_json::from_slice(&msg.payload) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to deserialize NATS message: {}", e);
                let _ = msg.ack().await;
                continue;
            }
        };

        match msg.subject.as_str() {
            FEED_CREATE_MARKET => {
                let market_id = match matcher_message.market_id {
                    Some(m) => m,
                    None => {
                        eprintln!("CREATE_MARKET message missing market field");
                        let _ = msg.ack().await;
                        continue;
                    }
                };

                channel_manager.create_market_channel(market_id).await;
                println!("Created channel for market {}", market_id);
            }

            FEED_REMOVE_MARKET => {
                let market_id = match matcher_message.market_id {
                    Some(m) => m,
                    None => {
                        eprintln!("REMOVE_MARKET message missing market field");
                        let _ = msg.ack().await;
                        continue;
                    }
                };

                channel_manager.remove_market_channel(market_id).await;
                println!("Removed channel for market {}", market_id);
            }

            FEED_MARKET_ORDER => {
                let order = match matcher_message.order {
                    Some(o) => o,
                    None => {
                        eprintln!("PLACE_ORDER message missing order field");
                        let _ = msg.ack().await;
                        continue;
                    }
                };

                // Broadcast the new order state to all subscribers of this market
                let server_message = ServerMessage::OrderFeed {
                    market_id: order.market_id,
                    outcome_id: order.outcome_id,
                    quantity: order.quantity,
                    price: order.price,
                };

                match serde_json::to_string(&server_message) {
                    Ok(text) => {
                        channel_manager
                            .broadcast_to_market(order.market_id, Message::Text(text.into()))
                            .await;
                    }
                    Err(e) => eprintln!("Failed to serialize OrderPlaced message: {}", e),
                }
            }

            unknown => {
                eprintln!("Unknown NATS subject: {}", unknown);
            }
        }

        if let Err(e) = msg.ack().await {
            eprintln!("NATS ack failed: {}", e);
        }
    }
}
