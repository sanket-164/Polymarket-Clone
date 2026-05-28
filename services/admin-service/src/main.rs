use std::sync::Arc;

use axum::http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use common::config::{JWTConfig, PGConfig, RedisConfig};
use common::database::client::PGClient;
use common::nats_handler::NatsHandler;
use deadpool_redis::{Config, Pool, Runtime};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use tower_http::cors::CorsLayer;

use crate::router::create_router;

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
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
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
            println!("Connected to database");
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

    let publisher = match NatsHandler::new("nats://localhost:4222").await {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to connect publisher: {e}");
            std::process::exit(1);
        }
    };

    let redis_pool = Config::from_url(RedisConfig::init().redis_url)
        .create_pool(Some(Runtime::Tokio1))
        .unwrap();

    let app_state = AppState {
        jwt_config,
        pg_client: pg_client.clone(),
        publisher: publisher,
        redis_pool,
    };

    let app = create_router(Arc::new(app_state.clone())).layer(cors.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", 4000))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap()
}
