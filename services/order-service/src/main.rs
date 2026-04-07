use axum::http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use common::config::{db::PGConfig, jwt::JWTConfig};
use common::database::client::PGClient;
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::{nats_publisher::Publisher, router::create_router};

pub mod db;
pub mod handler;
pub mod middleware;
pub mod nats_publisher;
pub mod router;

#[derive(Debug, Clone)]
pub struct AppState {
    pub jwt_config: JWTConfig,
    pub pg_client: PGClient,
    pub publisher: Publisher,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT]);

    let config = PGConfig::init();

    let pool = match PgPoolOptions::new()
        .max_connections(1)
        .connect(&config.database_url)
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
    let jwt_config = JWTConfig::init();

    let publisher = match Publisher::new("nats://localhost:4222").await {
        Ok(p) => p,
        Err(_e) => {
            println!("Failed to connect publisher");
            std::process::exit(1);
        }
    };

    let app_state = AppState {
        jwt_config,
        pg_client: pg_client.clone(),
        publisher: publisher,
    };

    let app = create_router(Arc::new(app_state.clone())).layer(cors.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", 5000))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap()
}
