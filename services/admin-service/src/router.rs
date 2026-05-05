use std::sync::Arc;

use axum::{Router, middleware, routing::get};
use common::constant::{API, AUTH, MARKET, PROFILE, ROOT};

use crate::{
    AppState,
    handler::{auth::auth_handler, health_check, market::market_handler, profile::profile_handler},
    middleware::auth_middleware,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .route(ROOT, get(health_check))
        .nest(AUTH, auth_handler())
        .nest(
            PROFILE,
            profile_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            MARKET,
            market_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .with_state(app_state);

    Router::new().nest(API, api_route)
}
