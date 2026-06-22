pub mod holding;
pub mod profile;
pub mod wallet;
use axum::response::IntoResponse;

pub async fn health_check() -> impl IntoResponse {
    "Server is running 🚀"
}
