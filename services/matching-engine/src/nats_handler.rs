use async_nats::{
    Client, ConnectOptions,
    jetstream::{self, consumer::pull::Stream},
};
use common::{
    constant::{FEED_MARKET_ORDER, MATCHER_STREAM, MAX_NATS_RECONNECTS, TRADE_UPDATE_ORDER},
    model::{FeedMessage, TradeMessage},
};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NatsHandler {
    pub client: Client,
    pub jetstream: jetstream::Context,
}

impl NatsHandler {
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

        let jetstream = jetstream::new(client.clone());

        Ok(Self { client, jetstream })
    }

    pub async fn get_matcher_stream(&self) -> Result<Stream, async_nats::Error> {
        let stream = self
            .jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: MATCHER_STREAM.into(),
                subjects: vec![format!("{MATCHER_STREAM}.>").into()],
                ..Default::default()
            })
            .await?;

        let consumer = stream
            .get_or_create_consumer(
                "pull-worker",
                jetstream::consumer::pull::Config {
                    durable_name: Some("pull-worker".into()),
                    ..Default::default()
                },
            )
            .await?;

        Ok(consumer.messages().await?)
    }

    pub async fn feed_market_order(&self, message: FeedMessage) -> Result<(), async_nats::Error> {
        let payload = serde_json::to_vec(&message).map_err(|e| {
            async_nats::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

        self.client
            .publish(FEED_MARKET_ORDER, payload.into())
            .await?;

        self.client.flush().await?;

        Ok(())
    }

    pub async fn trade_update_order(&self, message: TradeMessage) -> Result<(), async_nats::Error> {
        let payload = serde_json::to_vec(&message).map_err(|e| {
            async_nats::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

        self.jetstream
            .publish(TRADE_UPDATE_ORDER, payload.into())
            .await?
            .await?;
        Ok(())
    }
}
