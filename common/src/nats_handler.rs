use crate::{
    constant::{
        MAX_RECONNECTS, NATS_STREAM, SUBJECT_CENCEL_ORDER, SUBJECT_INSERT_MARKET,
        SUBJECT_INSERT_ORDER, SUBJECT_REMOVE_MARKET,
    },
    model::NatsMessage,
};
use async_nats::{
    ConnectOptions,
    jetstream::{self, consumer::pull::Stream},
};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NatsHandler {
    pub jetstream: jetstream::Context,
}

pub enum PublishMessage {
    InsertOrder,
    CancelOrder,
    InsertMarket,
    RemoveMarket,
}

impl NatsHandler {
    pub async fn new(url: &str) -> Result<Self, async_nats::Error> {
        let client = ConnectOptions::new()
            .max_reconnects(Some(MAX_RECONNECTS as usize))
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

    pub async fn get_message_stream(&self) -> Result<Stream, async_nats::Error> {
        let stream = self
            .jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: NATS_STREAM.into(),
                subjects: vec![format!("{}.>", NATS_STREAM.to_string()).into()],
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

        let message_stream = consumer.messages().await?;

        Ok(message_stream)
    }

    pub async fn publish_message(
        &self,
        message: NatsMessage,
        message_type: PublishMessage,
    ) -> Result<(), async_nats::Error> {
        let payload = serde_json::to_vec(&message).map_err(|e| {
            async_nats::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

        let message_subject;

        match message_type {
            PublishMessage::InsertOrder => {
                message_subject = SUBJECT_INSERT_ORDER;
            }
            PublishMessage::CancelOrder => {
                message_subject = SUBJECT_CENCEL_ORDER;
            }
            PublishMessage::InsertMarket => {
                message_subject = SUBJECT_INSERT_MARKET;
            }
            PublishMessage::RemoveMarket => {
                message_subject = SUBJECT_REMOVE_MARKET;
            }
        }

        self.jetstream
            .publish(message_subject, payload.into())
            .await?
            .await?;

        Ok(())
    }
}
