mod db;
mod engine;

use common::{
    config::PGConfig,
    constant::{
        SUBJECT_CENCEL_ORDER, SUBJECT_CREATE_MARKET, SUBJECT_PLACE_ORDER, SUBJECT_REMOVE_MARKET,
    },
    database::client::PGClient,
    model::NatsMessage,
    nats_handler::NatsHandler,
};
use futures::StreamExt;
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};

use crate::engine::Engine;

pub struct AppState {
    pub pg_client: PGClient,
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

    let mut engine = Engine::new();

    let m_id = uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").expect("lol");
    let fo_id = uuid::Uuid::parse_str("b2c3d4e5-f6a7-8901-bcde-f12345678901").expect("lol");
    let so_id = uuid::Uuid::parse_str("c3d4e5f6-a7b8-9012-cdef-123456789012").expect("lol");

    engine.add_market(m_id, fo_id, so_id);

    let nats_consumer = match NatsHandler::new("nats://localhost:4222").await {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to connect consumer: {e}");
            std::process::exit(1);
        }
    };

    let mut message_stream = nats_consumer
        .get_message_stream()
        .await
        .expect("Failed to get messages");

    while let Some(msg) = message_stream.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Message error: {e}");
                continue;
            }
        };

        let message: NatsMessage = match serde_json::from_slice(&msg.payload) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Deserialize error: {e}");
                let _ = msg.ack().await;
                continue;
            }
        };

        match msg.subject.as_str() {
            SUBJECT_PLACE_ORDER => {
                engine.match_order(message.order.unwrap(), &pg_client).await;
            }
            SUBJECT_CREATE_MARKET => {
                let market = message.market.expect("Market does not exist in message");
                let outcomes = message
                    .outcomes
                    .expect("Outcomes does not exist in message");
                engine.add_market(
                    market.id,
                    outcomes.first_outcome.id,
                    outcomes.second_outcome.id,
                );
            }
            SUBJECT_REMOVE_MARKET => {
                let market = message.market.expect("Market does not exist in message");
                engine.remove_market(market.id);
            }
            SUBJECT_CENCEL_ORDER => {
                // handle cancel order
            }
            unknown => {
                eprintln!("Unknown subject: {unknown}");
            }
        }

        if let Err(e) = msg.ack().await {
            eprintln!("Ack failed: {e}");
        }
    }
}
