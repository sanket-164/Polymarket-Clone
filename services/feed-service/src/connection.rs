use crate::handler::handle_connection;
use crate::manager::ChannelManager;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn start_websocket(
    addr: &str,
    channel_manager: Arc<ChannelManager>,
    allow_origins: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!(address = %addr, "Feed Service is listening");

    while let Ok((stream, address)) = listener.accept().await {
        let channel_manager = channel_manager.clone();
        let allow_origins = allow_origins.clone();

        tokio::spawn(async move {
            handle_connection(stream, address, channel_manager, allow_origins).await;
        });
    }

    Ok(())
}
