use crate::manager::ChannelManager;
use futures::{SinkExt, stream::StreamExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

pub async fn handle_connection(
    stream: TcpStream,
    address: SocketAddr,
    channel_manager: Arc<ChannelManager>,
) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during the websocket handshake");

    println!("New WebSocket connection: {}", address);

    let (mut write, mut read) = ws_stream.split();

    let (tx, mut rx) = mpsc::unbounded_channel();

    let channel_manager = channel_manager.clone();

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = write.send(message).await {
                eprintln!("Websocket send error: {}", e);
                break;
            }
        }
    });

    let mut current_channel = Uuid::nil();

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {}
            Err(e) => {
                eprintln!("Error processing message: {}", e);
                break;
            }
            Ok(_) => {}
        }
    }

    if !current_channel.is_nil() {
        channel_manager.leave_channel(current_channel, &tx).await;

        println!(
            "User {} disconnected and removed from room {}",
            address, current_channel
        );
    }
}
