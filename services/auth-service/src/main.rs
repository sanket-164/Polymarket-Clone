use std::sync::Arc;

use axum::http::{
    Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use common::database::client::PGClient;
use common::{
    config::{JWTConfig, PGConfig},
    constant::AUTH_PORT,
};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::router::create_router;

pub mod db;
pub mod handler;
pub mod router;

#[derive(Debug, Clone)]
pub struct AppState {
    pub jwt_config: JWTConfig,
    pub pg_client: PGClient,
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

    let app_state = AppState {
        jwt_config,
        pg_client,
    };

    let app = create_router(Arc::new(app_state.clone())).layer(cors.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", AUTH_PORT))
        .await
        .unwrap();

    println!("Auth Service is listening at http://localhost:{AUTH_PORT}");

    axum::serve(listener, app).await.unwrap()
}
