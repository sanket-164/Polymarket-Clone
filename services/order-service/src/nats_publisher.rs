use std::time::Duration;

use async_nats::{ConnectOptions, jetstream};

#[derive(Debug, Clone)]
pub struct Publisher {
    jetstream: jetstream::Context,
}

impl Publisher {
    pub async fn new(url: &str) -> Result<Self, async_nats::Error> {
        let client = ConnectOptions::new()
            .max_reconnects(5)
            .reconnect_delay_callback(|attempts| {
                Duration::from_millis(100 * 2_u64.pow(attempts as u32))
            })
            .event_callback(|event| async move {
                match event {
                    async_nats::Event::Disconnected => println!("Publisher Disconnected!"),
                    async_nats::Event::Connected => println!("Publisher Connected!"),
                    async_nats::Event::ClientError(e) => eprintln!("Publisher Client Error: {e}"),
                    async_nats::Event::ServerError(e) => eprintln!("Publisher Server Error: {e}"),
                    _ => {}
                }
            })
            .connect(url)
            .await?;

        Ok(Self {
            jetstream: jetstream::new(client),
        })
    }

    pub async fn publish(&self, stream_name: &str, message: &str) -> Result<(), async_nats::Error> {
        let _stream = self
            .jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: stream_name.into(),
                subjects: vec!["tasks.>".into()],
                ..Default::default()
            })
            .await?;

        let ack = self
            .jetstream
            .publish("tasks.order", message.to_string().into())
            .await?
            .await?; // Second .await confirms server ack

        println!("Published, seq: {}", ack.sequence);

        Ok(())
    }
}
