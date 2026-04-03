pub mod order;
use axum::response::IntoResponse;

pub async fn health_check() -> impl IntoResponse {
    "Order service is running 🚀"
}
