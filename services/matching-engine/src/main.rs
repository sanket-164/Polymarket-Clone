mod db;
mod engine;
mod nats_consumer;

use common::{
    config::PGConfig, constant::ORDER_STREAM, database::client::PGClient, model::market::Order,
};
use futures::StreamExt;
use nats_consumer::Consumer;
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};

use crate::engine::Engine;

pub struct AppState {
    pub pg_client: PGClient,
    pub consumer: Consumer,
}

#[tokio::main]
async fn main() {
    let pg_config = PGConfig::init();

    let pool = match PgPoolOptions::new()
        .max_connections(pg_config.pool_size_each_service)
        .connect(&pg_config.database_url)
        .await
    {
        Ok(pool) => {
            println!("Connected to database");
            pool
        }
        Err(_err) => {
            println!("Failed to connect to database");
            // Fail fast: Application cannot run without DB
            std::process::exit(1);
        }
    };

    static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

    MIGRATOR.run(&pool).await.expect("Failed to run migrations");

    let pg_client = PGClient::new(pool);

    let consumer = match Consumer::new("nats://localhost:4222").await {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to connect consumer: {e}");
            std::process::exit(1);
        }
    };

    let mut engine = Engine::new();

    let m_id = uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").expect("lol");
    let fo_id = uuid::Uuid::parse_str("b2c3d4e5-f6a7-8901-bcde-f12345678901").expect("lol");
    let so_id = uuid::Uuid::parse_str("c3d4e5f6-a7b8-9012-cdef-123456789012").expect("lol");

    engine.add_market(m_id, fo_id, so_id);

    let stream = consumer
        .jetstream
        .get_stream(ORDER_STREAM)
        .await
        .expect("Failed to get stream");

    let consumer_handle = stream
        .get_or_create_consumer(
            "matching-engine",
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("matching-engine".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create consumer");

    let mut messages = consumer_handle
        .messages()
        .await
        .expect("Failed to get messages");

    while let Some(msg) = messages.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Message error: {e}");
                continue;
            }
        };

        let order: Order = match serde_json::from_slice(&msg.payload) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Deserialize error: {e}");
                let _ = msg.ack().await;
                continue;
            }
        };

        {
            engine.match_order(order, &pg_client).await;
        }

        if let Err(e) = msg.ack().await {
            eprintln!("Ack failed: {e}");
        }
    }
}
