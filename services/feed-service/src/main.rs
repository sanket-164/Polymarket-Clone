mod connection;
mod consumer;
mod handler;
mod manager;
mod nats_handler;

use common::{config::NatsConfig, constant::FEED_PORT};
use manager::ChannelManager;
use std::sync::Arc;

use crate::nats_handler::NatsClient;

#[tokio::main]
async fn main() {
    let nats_client = match NatsClient::new(&NatsConfig::init().nats_url).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to NATS: {}", e);
            std::process::exit(1);
        }
    };
    let nats_client = Arc::new(nats_client);

    let channel_manager = Arc::new(ChannelManager::new());

    // Spawn NATS consumer loop
    tokio::spawn(consumer::start_consumer(
        nats_client.clone(),
        channel_manager.clone(),
    ));

    // Run WebSocket server (blocks)
    connection::start_websocket(&format!("0.0.0.0:{FEED_PORT}"), channel_manager)
        .await
        .unwrap();
}
