use std::sync::Arc;

use axum::{Router, middleware, routing::get};
use common::constant::{API_PREFIX, HEALTH_CHECK, ORDER_PREFIX};

use crate::{
    AppState,
    handler::{health_check, order::order_handler},
    middleware::auth_middleware,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .route(HEALTH_CHECK, get(health_check))
        .nest(
            ORDER_PREFIX,
            order_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .with_state(app_state);

    Router::new().nest(API_PREFIX, api_route)
}
