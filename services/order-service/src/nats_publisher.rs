use std::time::Duration;

use async_nats::{ConnectOptions, jetstream};
use common::{
    constant::{MAX_RECONNECTS, ORDER_STREAM},
    model::market::Order,
};

#[derive(Debug, Clone)]
pub struct Publisher {
    jetstream: jetstream::Context,
}

impl Publisher {
    pub async fn new(url: &str) -> Result<Self, async_nats::Error> {
        let client = ConnectOptions::new()
            .max_reconnects(Some(MAX_RECONNECTS as usize))
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

    pub async fn publish(&self, message: Order) -> Result<(), async_nats::Error> {
        let _stream = self
            .jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: ORDER_STREAM.into(),
                subjects: vec!["order.>".into()],
                ..Default::default()
            })
            .await?;

        let payload = serde_json::to_vec(&message).map_err(|e| {
            async_nats::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

        self.jetstream
            .publish("order.insert", payload.into())
            .await?
            .await?;

        println!("Published order: {}", message.id);

        Ok(())
    }
}
