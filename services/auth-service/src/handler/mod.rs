pub mod admin;
pub mod user;

use axum::response::IntoResponse;

pub async fn health_check() -> impl IntoResponse {
    "Server is running 🚀"
}
