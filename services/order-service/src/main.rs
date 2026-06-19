use axum::http::{
    Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use common::{
    config::{JWTConfig, PGConfig},
    constant::ORDER_PORT,
    nats_handler::NatsHandler,
};
use common::{
    config::{NatsConfig, RedisConfig},
    database::client::PGClient,
};
use deadpool_redis::{Config, Pool, Runtime};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::router::create_router;

pub mod consumer;
pub mod db;
pub mod handler;
pub mod middleware;
pub mod router;

#[derive(Debug, Clone)]
pub struct AppState {
    pub jwt_config: JWTConfig,
    pub pg_client: PGClient,
    pub publisher: NatsHandler,
    pub redis_pool: Pool,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT]);

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
    let jwt_config = JWTConfig::init();

    let publisher = match NatsHandler::new(&NatsConfig::init().nats_url).await {
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

    tokio::spawn(consumer::start_consumer(
        Arc::new(publisher.clone()),
        Arc::new(pg_client.clone()),
        Arc::new(redis_pool.clone()),
    ));

    let app_state = AppState {
        jwt_config,
        pg_client,
        publisher,
        redis_pool,
    };

    let app = create_router(Arc::new(app_state.clone())).layer(cors.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", ORDER_PORT))
        .await
        .unwrap();

    println!("Order Service is listening at http://localhost:{ORDER_PORT}");

    axum::serve(listener, app).await.unwrap()
}
