use crate::{
    constant::{
        FEED_CREATE_MARKET, FEED_MARKET_ORDER, FEED_REMOVE_MARKET, MATCHER_CANCEL_ORDER,
        MATCHER_CREATE_MARKET, MATCHER_PLACE_ORDER, MATCHER_REMOVE_MARKET, MAX_NATS_RECONNECTS,
        TRADE_UPDATE_ORDER,
    },
    model::{FeedMessage, MatcherMessage, TradeMessage},
};
use async_nats::{
    ConnectOptions,
    jetstream::{self, consumer::pull::Stream},
};
use serde::Serialize;
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

        Ok(Self {
            jetstream: jetstream::new(client),
        })
    }

    pub async fn get_message_stream(&self, stream_name: &str) -> Result<Stream, async_nats::Error> {
        let stream = self
            .jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: stream_name.into(),
                subjects: vec![format!("{stream_name}.>").into()],
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

    // Single generic publish function
    async fn publish<T: Serialize>(
        &self,
        subject: &'static str,
        message: &T,
    ) -> Result<(), async_nats::Error> {
        let payload = serde_json::to_vec(message).map_err(|e| {
            async_nats::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

        self.jetstream
            .publish(subject, payload.into())
            .await?
            .await?;
        Ok(())
    }

    pub async fn matcher_place_order(
        &self,
        message: MatcherMessage,
    ) -> Result<(), async_nats::Error> {
        self.publish(MATCHER_PLACE_ORDER, &message).await
    }

    pub async fn matcher_cancel_order(
        &self,
        message: MatcherMessage,
    ) -> Result<(), async_nats::Error> {
        self.publish(MATCHER_CANCEL_ORDER, &message).await
    }

    pub async fn matcher_create_market(
        &self,
        message: MatcherMessage,
    ) -> Result<(), async_nats::Error> {
        self.publish(MATCHER_CREATE_MARKET, &message).await
    }

    pub async fn matcher_remove_market(
        &self,
        message: MatcherMessage,
    ) -> Result<(), async_nats::Error> {
        self.publish(MATCHER_REMOVE_MARKET, &message).await
    }

    pub async fn feed_market_order(&self, message: FeedMessage) -> Result<(), async_nats::Error> {
        self.publish(FEED_MARKET_ORDER, &message).await
    }

    pub async fn feed_create_market(&self, message: FeedMessage) -> Result<(), async_nats::Error> {
        self.publish(FEED_CREATE_MARKET, &message).await
    }

    pub async fn feed_remove_market(&self, message: FeedMessage) -> Result<(), async_nats::Error> {
        self.publish(FEED_REMOVE_MARKET, &message).await
    }

    pub async fn trade_update_order(&self, message: TradeMessage) -> Result<(), async_nats::Error> {
        self.publish(TRADE_UPDATE_ORDER, &message).await
    }
}
