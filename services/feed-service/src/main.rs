mod connection;
mod handler;
mod manager;

#[tokio::main]
async fn main() {
    connection::start_websocket("0.0.0.0:5000").await.unwrap();
}
