use common::constant::RESOLVE_STREAM;
use common::model::{FeedMessage, MatcherMessage, ResolveMessage};
use common::{config::NatsConfig, database::client::PGClient};
use common::{config::PGConfig, nats_handler::NatsHandler};
use futures::StreamExt;
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};

use crate::db::MarketExt;

pub mod db;

#[tokio::main]
async fn main() {
    let pg_config = PGConfig::init();

    let pool = match PgPoolOptions::new()
        .max_connections(pg_config.pool_size_each_service)
        .connect(&pg_config.database_url)
        .await
    {
        Ok(pool) => {
            println!("Database Connected!");
            pool
        }
        Err(e) => {
            println!("Failed to connect to database: {e}");
            // Fail fast: Application cannot run without DB
            std::process::exit(1);
        }
    };

    static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

    MIGRATOR.run(&pool).await.expect("Failed to run migrations");

    let pg_client = PGClient::new(pool);

    let nats_handler = match NatsHandler::new(&NatsConfig::init().nats_url).await {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to connect publisher: {e}");
            std::process::exit(1);
        }
    };

    let mut message_stream = nats_handler
        .get_message_stream(RESOLVE_STREAM)
        .await
        .expect("Failed to get messages");

    println!("Resolve Consumer is ready to receive message");

    while let Some(msg) = message_stream.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Message error: {e}");
                continue;
            }
        };

        let message: ResolveMessage = match serde_json::from_slice(&msg.payload) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Deserialize error: {e}");
                let _ = msg.ack().await;
                continue;
            }
        };

        match message {
            ResolveMessage::ResolveMarket {
                market_id,
                winning_outcome_id,
            } => {
                match pg_client
                    .resolve_market(market_id, winning_outcome_id)
                    .await
                {
                    Err(e) => {
                        eprintln!("Deserialize error: {e}");
                        let _ = msg.ack().await;
                        continue;
                    }
                    Ok(_) => {
                        if let Err(e) = nats_handler
                            .feed_remove_market(FeedMessage::RemoveMarket { market_id })
                            .await
                        {
                            eprintln!("Feed remove market error: {e}");
                        }

                        if let Err(e) = nats_handler
                            .matcher_remove_market(MatcherMessage::RemoveMarket { market_id })
                            .await
                        {
                            eprintln!("Feed remove market error: {e}");
                        }
                    }
                }
            }
        }

        if let Err(e) = msg.ack().await {
            eprintln!("Ack failed: {e}");
        }
    }
}
