use async_nats::{ConnectOptions, jetstream};
use futures::StreamExt;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Consumer {
    jetstream: jetstream::Context,
}

impl Consumer {
    pub async fn new(url: &str) -> Result<Self, async_nats::Error> {
        let client = ConnectOptions::new()
            .max_reconnects(5)
            .reconnect_delay_callback(|attempts| {
                Duration::from_millis(100 * 2_u64.pow(attempts as u32))
            })
            .event_callback(|event| async move {
                match event {
                    async_nats::Event::Disconnected => println!("Consumer Disconnected!"),
                    async_nats::Event::Connected => println!("Consumer Connected!"),
                    async_nats::Event::ClientError(e) => eprintln!("Consumer Client Error: {e}"),
                    async_nats::Event::ServerError(e) => eprintln!("Consumer Server Error: {e}"),
                    _ => {}
                }
            })
            .connect(url)
            .await?;

        Ok(Self {
            jetstream: jetstream::new(client),
        })
    }

    pub async fn consume(&self, stream_name: &str) -> Result<(), async_nats::Error> {
        let stream = self
            .jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: stream_name.into(),
                subjects: vec!["tasks.>".into()],
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

        let mut messages = consumer.messages().await?;

        while let Some(Ok(msg)) = messages.next().await {
            println!("Got: {}", std::str::from_utf8(&msg.payload)?);
            msg.ack().await?;
        }

        Ok(())
    }
}
