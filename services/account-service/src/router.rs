use std::sync::Arc;

use axum::{Router, middleware, routing::get};

use crate::{
    AppState,
    handler::{auth::auth_handler, health_check, profile::profile_handler, wallet::wallet_handler},
    middleware::auth_middleware,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .route("/", get(health_check))
        .nest("/auth", auth_handler())
        .nest(
            "/profile",
            profile_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            "/wallet",
            wallet_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .with_state(app_state);

    Router::new().nest("/api", api_route)
}
