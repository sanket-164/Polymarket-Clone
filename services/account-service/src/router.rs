use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{AppState, handler::health_check};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new().route("/", get(health_check));

    Router::new().nest("/api", api_route)
}
