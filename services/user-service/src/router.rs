use std::sync::Arc;

use axum::{Router, middleware, routing::get};
use common::constant::{API, HOLDING, PROFILE, ROOT, USER, WALLET};

use crate::{
    AppState,
    handler::{
        health_check, holding::holding_handler, profile::profile_handler, wallet::wallet_handler,
    },
    middleware::auth_middleware,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .route(ROOT, get(health_check))
        .nest(
            PROFILE,
            profile_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            WALLET,
            wallet_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            HOLDING,
            holding_handler().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .with_state(app_state);

    Router::new().nest(&format!("{API}{USER}"), api_route)
}
