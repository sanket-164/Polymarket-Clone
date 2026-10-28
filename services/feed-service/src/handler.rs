use crate::manager::ChannelManager;
use futures::{SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        Message,
        handshake::server::{Request, Response as HandshakeResponse},
        http::{
            HeaderValue, Response, StatusCode,
            header::{ACCESS_CONTROL_ALLOW_ORIGIN, ORIGIN},
        },
    },
};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    JoinMarket { market_id: Uuid },
    LeaveMarket { market_id: Uuid },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ServerMessage {
    JoinedMarket { market_id: Uuid },
    LeftMarket { market_id: Uuid },
    Error { message: String },
}

pub async fn handle_connection(
    stream: TcpStream,
    address: SocketAddr,
    channel_manager: Arc<ChannelManager>,
    allow_origins: String,
) {
    let ws_stream = match accept_hdr_async(
        stream,
        |request: &Request, mut response: HandshakeResponse| {
            let origin = request.headers().get(ORIGIN);
            let origin_allowed = origin
                .map(|origin| {
                    allow_origins == "*"
                        || allow_origins
                            .split(',')
                            .map(str::trim)
                            .any(|allowed| allowed.as_bytes() == origin.as_bytes())
                })
                .unwrap_or(true);

            if !origin_allowed {
                tracing::warn!(
                    ?address,
                    ?origin,
                    "Rejected WebSocket connection from disallowed origin"
                );
                let response = Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Some("origin is not allowed".to_string()))
                    .expect("valid WebSocket rejection response");
                return Err(response);
            }

            if let Some(origin) = origin {
                response.headers_mut().insert(
                    ACCESS_CONTROL_ALLOW_ORIGIN,
                    HeaderValue::from_bytes(origin.as_bytes())
                        .expect("WebSocket Origin header must be a valid header value"),
                );
            }

            Ok(response)
        },
    )
    .await
    {
        Ok(ws_stream) => ws_stream,
        Err(error) => {
            tracing::warn!(?address, %error, "WebSocket handshake failed");
            return;
        }
    };

    tracing::info!(%address, "New WebSocket connection");

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
            Ok(Message::Text(text)) => {
                let msg = serde_json::from_str(&text);

                if msg.is_err() {
                    let error_message = ServerMessage::Error {
                        message: msg.err().unwrap().to_string(),
                    };

                    if let Ok(text) = serde_json::to_string(&error_message) {
                        if let Err(e) = tx.send(Message::Text(text.into())) {
                            eprintln!("Failed to send message: {}", e);
                        }
                    }
                    continue;
                }

                let client_message: ClientMessage = msg.unwrap();

                match client_message {
                    ClientMessage::JoinMarket { market_id } => {
                        if !current_channel.is_nil() {
                            channel_manager
                                .leave_market_channel(current_channel, &tx.clone())
                                .await;
                        }

                        channel_manager
                            .join_market_channel(market_id, tx.clone())
                            .await;

                        current_channel = market_id;

                        let joined_message = ServerMessage::JoinedMarket { market_id };

                        if let Ok(text) = serde_json::to_string(&joined_message) {
                            if let Err(e) = tx.send(Message::Text(text.into())) {
                                eprintln!("Failed to send message: {}", e);
                            }
                        }
                    }

                    ClientMessage::LeaveMarket { market_id } => {
                        if !current_channel.eq(&market_id) || current_channel.is_nil() {
                            let error_message = ServerMessage::Error {
                                message: format!(
                                    "You are not in channel of market_id: {market_id}"
                                ),
                            };

                            if let Ok(text) = serde_json::to_string(&error_message) {
                                if let Err(e) = tx.send(Message::Text(text.into())) {
                                    eprintln!("Failed to send message: {}", e);
                                }
                            }

                            continue;
                        }

                        let market_channel = channel_manager.get_market_channel(market_id).await;

                        if market_channel.is_none() {
                            let error_message = ServerMessage::Error {
                                message: format!(
                                    "Channel for the market_id: {market_id} does not exist"
                                ),
                            };

                            if let Ok(text) = serde_json::to_string(&error_message) {
                                if let Err(e) = tx.send(Message::Text(text.into())) {
                                    eprintln!("Failed to send message: {}", e);
                                }
                            }

                            continue;
                        }

                        channel_manager
                            .leave_market_channel(market_id, &tx.clone())
                            .await;

                        current_channel = Uuid::nil();

                        let left_message = ServerMessage::LeftMarket { market_id };

                        if let Ok(text) = serde_json::to_string(&left_message) {
                            if let Err(e) = tx.send(Message::Text(text.into())) {
                                eprintln!("Failed to send message: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error processing message: {}", e);
                break;
            }
            Ok(_) => {}
        }
    }

    if !current_channel.is_nil() {
        channel_manager
            .leave_market_channel(current_channel, &tx)
            .await;

        println!(
            "User {} disconnected and removed from room {}",
            address, current_channel
        );
    }
}
