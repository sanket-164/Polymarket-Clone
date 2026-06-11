pub mod market;
pub mod profile;

use axum::response::IntoResponse;

pub async fn health_check() -> impl IntoResponse {
    "Server is running 🚀"
}
