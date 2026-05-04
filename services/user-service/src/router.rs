use std::sync::Arc;

use axum::{Router, middleware, routing::get};
use common::constant::{API_PREFIX, AUTH_PREFIX, HEALTH_CHECK, PROFILE_PREFIX, WALLET_PREFIX};

use crate::{
    AppState,
    handler::{auth::auth_handler, health_check, profile::profile_handler, wallet::wallet_handler},
    middleware::auth_middleware,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .route(HEALTH_CHECK, get(health_check))
        .nest(AUTH_PREFIX, auth_handler())
        .nest(
            PROFILE_PREFIX,
            profile_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            WALLET_PREFIX,
            wallet_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .with_state(app_state);

    Router::new().nest(API_PREFIX, api_route)
}
