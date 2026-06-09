mod db;
mod engine;

use common::{
    config::{PGConfig, RedisConfig},
    constant::MATCHER_STREAM,
    database::client::PGClient,
    model::MatcherMessage,
    nats_handler::NatsHandler,
};
use deadpool_redis::{Config, Runtime};
use futures::StreamExt;
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};

use crate::engine::Engine;

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

    let mut redis_connection = Config::from_url(RedisConfig::init().redis_url)
        .create_pool(Some(Runtime::Tokio1))
        .unwrap()
        .get()
        .await
        .unwrap();

    let mut engine = Engine::new();

    let nats_handler = match NatsHandler::new("nats://localhost:4222").await {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to connect nats handler: {e}");
            std::process::exit(1);
        }
    };

    let mut message_stream = nats_handler
        .get_message_stream(MATCHER_STREAM)
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

        let message: MatcherMessage = match serde_json::from_slice(&msg.payload) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Deserialize error: {e}");
                let _ = msg.ack().await;
                continue;
            }
        };

        match message {
            MatcherMessage::PlaceOrder { order } => {
                engine
                    .match_order(order, &pg_client, &mut redis_connection, &nats_handler)
                    .await;
            }
            MatcherMessage::CreateMarket { market, outcomes } => {
                engine.add_market(
                    market.id,
                    outcomes.first_outcome.id,
                    outcomes.second_outcome.id,
                );
            }
            MatcherMessage::RemoveMarket { market } => {
                engine.remove_market(market.id);
            }
            MatcherMessage::CancelOrder { order: _order } => {
                // handle cancel order
            }
        }

        if let Err(e) = msg.ack().await {
            eprintln!("Ack failed: {e}");
        }
    }
}
