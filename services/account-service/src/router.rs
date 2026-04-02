use std::sync::Arc;

use axum::{Router, middleware, routing::get};

use crate::{
    AppState,
    handler::{auth::auth_handler, health_check, profile::profile_handler},
    middleware::auth,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .route("/", get(health_check))
        .nest("/auth", auth_handler())
        .nest(
            "/profile",
            profile_handler().layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state);

    Router::new().nest("/api", api_route)
}
