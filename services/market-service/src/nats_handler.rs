use async_nats::{
    Client, ConnectOptions,
    jetstream::{self, consumer::pull::Stream},
};
use common::{
    constant::{
        FEED_CREATE_MARKET, FEED_REMOVE_MARKET, MATCHER_CREATE_MARKET, MATCHER_PLACE_ORDER,
        MATCHER_REMOVE_MARKET, MAX_NATS_RECONNECTS,
    },
    model::{FeedMessage, MatcherMessage},
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

    async fn publish_jetstream(
        &self,
        subject: &'static str,
        message: &MatcherMessage,
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

    async fn publish_queue(
        &self,
        subject: &'static str,
        message: &FeedMessage,
    ) -> Result<(), async_nats::Error> {
        let payload = serde_json::to_vec(&message).map_err(|e| {
            async_nats::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

        self.client.publish(subject, payload.into()).await?;

        self.client.flush().await?;

        Ok(())
    }

    pub async fn matcher_place_order(
        &self,
        message: MatcherMessage,
    ) -> Result<(), async_nats::Error> {
        self.publish_jetstream(MATCHER_PLACE_ORDER, &message).await
    }

    pub async fn matcher_create_market(
        &self,
        message: MatcherMessage,
    ) -> Result<(), async_nats::Error> {
        self.publish_jetstream(MATCHER_CREATE_MARKET, &message)
            .await
    }

    pub async fn matcher_remove_market(
        &self,
        message: MatcherMessage,
    ) -> Result<(), async_nats::Error> {
        self.publish_jetstream(MATCHER_REMOVE_MARKET, &message)
            .await
    }

    pub async fn feed_create_market(&self, message: FeedMessage) -> Result<(), async_nats::Error> {
        self.publish_queue(FEED_CREATE_MARKET, &message).await
    }

    pub async fn feed_remove_market(&self, message: FeedMessage) -> Result<(), async_nats::Error> {
        self.publish_queue(FEED_REMOVE_MARKET, &message).await
    }
}
