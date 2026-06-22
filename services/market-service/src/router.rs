use std::sync::Arc;

use axum::{Router, middleware, routing::get};
use common::constant::{API, MARKET, ROOT};

use crate::{
    AppState,
    handler::{
        health_check,
        market::{market_handler, public_market_handler},
    },
    middleware::auth_middleware,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .route(ROOT, get(health_check))
        .nest(
            MARKET,
            market_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(MARKET, public_market_handler()) // no auth layer
        .with_state(app_state);

    Router::new().nest(API, api_route)
}
