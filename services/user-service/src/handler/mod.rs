pub mod auth;
pub mod holding;
pub mod market;
pub mod profile;
pub mod wallet;
use axum::response::IntoResponse;

pub async fn health_check() -> impl IntoResponse {
    "Server is running 🚀"
}
