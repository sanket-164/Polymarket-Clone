use async_nats::{
    ConnectOptions,
    jetstream::{self, consumer::pull::Stream},
};
use common::{
    constant::{
        MATCHER_STREAM, MAX_NATS_RECONNECTS, TRADE_CANCEL_ORDER, TRADE_COMPLETE_ORDER,
        TRADE_LIMIT_ORDER, TRADE_MARKET_ORDER,
    },
    model::TradeMessage,
};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NatsHandler {
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

        Ok(Self { jetstream })
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

    pub async fn trade_limit_order(&self, message: TradeMessage) -> Result<(), async_nats::Error> {
        let payload = serde_json::to_vec(&message).map_err(|e| {
            async_nats::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

        self.jetstream
            .publish(TRADE_LIMIT_ORDER, payload.into())
            .await?
            .await?;
        Ok(())
    }

    pub async fn trade_cancel_order(&self, message: TradeMessage) -> Result<(), async_nats::Error> {
        let payload = serde_json::to_vec(&message).map_err(|e| {
            async_nats::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

        self.jetstream
            .publish(TRADE_CANCEL_ORDER, payload.into())
            .await?
            .await?;
        Ok(())
    }

    pub async fn trade_market_order(&self, message: TradeMessage) -> Result<(), async_nats::Error> {
        let payload = serde_json::to_vec(&message).map_err(|e| {
            async_nats::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

        self.jetstream
            .publish(TRADE_MARKET_ORDER, payload.into())
            .await?
            .await?;
        Ok(())
    }

    pub async fn trade_complete_order(
        &self,
        message: TradeMessage,
    ) -> Result<(), async_nats::Error> {
        let payload = serde_json::to_vec(&message).map_err(|e| {
            async_nats::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

        self.jetstream
            .publish(TRADE_COMPLETE_ORDER, payload.into())
            .await?
            .await?;
        Ok(())
    }
}
