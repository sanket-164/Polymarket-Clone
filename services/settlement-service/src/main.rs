use common::{
    config::{NatsConfig, PGConfig, RedisConfig},
    constant::RESOLVE_STREAM,
    database::client::PGClient,
    model::{FeedMessage, MatcherMessage, ResolveMessage},
    nats_handler::NatsHandler,
};
use deadpool_redis::{Config, Runtime, redis::AsyncCommands};
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

    let redis_pool = Config::from_url(RedisConfig::init().redis_url)
        .create_pool(Some(Runtime::Tokio1))
        .unwrap();

    println!("Redis Pool Created!");

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
                        let mut redis = redis_pool
                            .get()
                            .await
                            .expect("Cannot create redis connection");

                        let pattern = format!("orderbook:{}:*", market_id);

                        let keys: Vec<String> = {
                            let mut iter = match redis.scan_match::<_, String>(&pattern).await {
                                Ok(result) => result,
                                Err(e) => {
                                    eprintln!("Redis SCAN error: {e}");
                                    break;
                                }
                            };

                            let mut keys = Vec::new();
                            while let Some(key) = iter.next_item().await {
                                keys.push(key);
                            }
                            keys
                        }; // iter is dropped here, releasing the mutable borrow on redis

                        if !keys.is_empty() {
                            if let Err(e) = redis.del::<&Vec<String>, ()>(&keys).await {
                                eprintln!("Redis Orderbook DEL error: {e}");
                            }
                        }

                        let market_cache_key = format!("market:{}", market_id);

                        if let Err(e) = redis.del::<&String, ()>(&market_cache_key).await {
                            eprintln!("Redis Market DEL error: {e}");
                        }

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
