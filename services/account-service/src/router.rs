use std::sync::Arc;

use axum::{Extension, Router, routing::get};

use crate::{
    AppState,
    handler::{auth::auth_handler, health_check},
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .route("/", get(health_check))
        .nest("/auth", auth_handler())
        .layer(Extension(app_state));

    Router::new().nest("/api", api_route)
}
