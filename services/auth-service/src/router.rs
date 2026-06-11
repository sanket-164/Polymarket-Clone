use std::sync::Arc;

use axum::{Router, routing::get};
use common::constant::{ADMIN, API, ROOT, USER};

use crate::{
    AppState,
    handler::{admin::admin_auth_handler, health_check, user::user_auth_handler},
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .route(ROOT, get(health_check))
        .nest(ADMIN, admin_auth_handler())
        .nest(USER, user_auth_handler())
        .with_state(app_state);

    Router::new().nest(API, api_route)
}
