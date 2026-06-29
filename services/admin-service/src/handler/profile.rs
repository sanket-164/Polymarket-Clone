use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
};
use common::{
    constant::ROOT,
    error::{ErrorMessage, HttpError},
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    db::AccountExt,
    dto::{AdminResponse, UpdateAdminDTO},
};

pub fn profile_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route(ROOT, get(get_me))
        .route(ROOT, put(update_admin_profile))
}

async fn get_me(
    State(app_state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    let admin = app_state
        .pg_client
        .get_admin_by_id(admin_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(ErrorMessage::UserNotFound.to_string()))?;

    Ok((StatusCode::OK, Json(AdminResponse::from(admin))))
}

async fn update_admin_profile(
    State(app_state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Json(body): Json<UpdateAdminDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let updated_admin = app_state
        .pg_client
        .update_admin(admin_id, body.name, body.email)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::OK, Json(AdminResponse::from(updated_admin))))
}
