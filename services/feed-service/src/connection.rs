use crate::handler::handle_connection;
use crate::manager::ChannelManager;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn start_websocket(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    let channel_manager = Arc::new(ChannelManager::new());

    println!("Websocket Server running on {}", addr);

    while let Ok((stream, address)) = listener.accept().await {
        let channel_manager = channel_manager.clone();

        tokio::spawn(async move {
            handle_connection(stream, address, channel_manager).await;
        });
    }

    Ok(())
}
