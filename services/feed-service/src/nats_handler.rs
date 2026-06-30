use async_nats::{Client, ConnectOptions, Subscriber};
use common::constant::{FEED_QUEUE, MAX_NATS_RECONNECTS};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NatsClient {
    pub client: Client,
}

impl NatsClient {
    pub async fn new(url: &str) -> Result<Self, async_nats::Error> {
        let client = ConnectOptions::new()
            .max_reconnects(Some(MAX_NATS_RECONNECTS as usize))
            .reconnect_delay_callback(|attempts| {
                Duration::from_millis(100 * 2_u64.pow(attempts as u32))
            })
            .event_callback(|event| async move {
                match event {
                    async_nats::Event::Disconnected => println!("NATS Disconnected!"),
                    async_nats::Event::Connected => println!("NATS Connected!"),
                    async_nats::Event::ClientError(e) => eprintln!("NATS Client Error: {e}"),
                    async_nats::Event::ServerError(e) => eprintln!("NATS Server Error: {e}"),
                    _ => {}
                }
            })
            .connect(url)
            .await?;

        Ok(Self { client })
    }

    pub async fn get_feed_subscriber(&self) -> Result<Subscriber, async_nats::Error> {
        let subscriber = self
            .client
            .subscribe(format!("{FEED_QUEUE}.>").to_string())
            .await?;

        Ok(subscriber)
    }
}
