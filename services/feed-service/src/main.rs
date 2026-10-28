mod connection;
mod consumer;
mod handler;
mod manager;
mod nats_handler;

use common::{
    config::{NatsConfig, ServerConfig},
    constant::FEED_PORT,
};
use manager::ChannelManager;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::nats_handler::NatsClient;

#[tokio::main]
async fn main() {
    let server_config = ServerConfig::init();

    match server_config.environment.as_str() {
        "production" => {
            tracing_subscriber::registry()
                .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
                .with(
                    fmt::layer()
                        .json()
                        .with_current_span(true)
                        .with_span_list(true)
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_ansi(false),
                )
                .init();

            tracing::info!(env = "production", "Tracer initialized");
        }
        _ => {
            tracing_subscriber::registry()
                .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")))
                .with(
                    fmt::layer()
                        .pretty()
                        .with_target(true)
                        .with_thread_ids(false)
                        .with_ansi(true),
                )
                .init();

            tracing::debug!(env = "development", "Tracer initialized");
        }
    }

    let nats_client = match NatsClient::new(&NatsConfig::init().nats_url).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to connect to NATS");
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

    let service_port = server_config.port.unwrap_or(FEED_PORT.to_string());

    // Run WebSocket server (blocks)
    connection::start_websocket(
        &format!("0.0.0.0:{service_port}"),
        channel_manager,
        server_config.allow_origins,
    )
    .await
    .unwrap();
}
