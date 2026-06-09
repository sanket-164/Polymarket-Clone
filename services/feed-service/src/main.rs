mod connection;
mod consumer;
mod handler;
mod manager;

use common::nats_handler::NatsHandler;
use manager::ChannelManager;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let nats_handler = match NatsHandler::new("nats://localhost:4222").await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to NATS: {}", e);
            std::process::exit(1);
        }
    };
    let nats_handler = Arc::new(nats_handler);

    let channel_manager = Arc::new(ChannelManager::new());

    // Spawn NATS consumer loop
    tokio::spawn(consumer::start_consumer(
        nats_handler.clone(),
        channel_manager.clone(),
    ));

    // Run WebSocket server (blocks)
    connection::start_websocket("0.0.0.0:6000", channel_manager)
        .await
        .unwrap();
}
