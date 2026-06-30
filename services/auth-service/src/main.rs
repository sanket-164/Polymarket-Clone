use std::sync::Arc;

use axum::http::{
    Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use common::{
    config::{JWTConfig, PGConfig, RedisConfig, ServerConfig, SmtpConfig},
    constant::AUTH_PORT,
    database::client::PGClient,
};
use deadpool_redis::{Config, Pool, Runtime};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::router::create_router;

pub mod db;
pub mod dto;
pub mod handler;
pub mod router;

#[derive(Debug, Clone)]
pub struct AppState {
    pub jwt_config: JWTConfig,
    pub pg_client: PGClient,
    pub smtp_config: SmtpConfig,
    pub redis_pool: Pool,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT]);

    match ServerConfig::init().environment.as_str() {
        "production" => {
            tracing_subscriber::registry()
                .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
                .with(
                    fmt::layer()
                        .json()
                        .with_current_span(true)
                        .with_span_list(true)
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_ansi(false),
                )
                .init();

            tracing::info!(env = "production", "Tracer initialized");
        }
        _ => {
            tracing_subscriber::registry()
                .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")))
                .with(
                    fmt::layer()
                        .pretty()
                        .with_target(true)
                        .with_thread_ids(false)
                        .with_ansi(true),
                )
                .init();

            tracing::debug!(env = "development", "Tracer initialized");
        }
    }

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
    let smtp_config = SmtpConfig::init();

    let redis_pool = Config::from_url(RedisConfig::init().redis_url)
        .create_pool(Some(Runtime::Tokio1))
        .unwrap();

    let app_state = AppState {
        jwt_config,
        pg_client,
        smtp_config,
        redis_pool,
    };

    let app = create_router(Arc::new(app_state.clone())).layer(cors.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{AUTH_PORT}"))
        .await
        .unwrap();

    println!("Auth Service is listening at http://localhost:{AUTH_PORT}");

    axum::serve(listener, app).await.unwrap()
}
