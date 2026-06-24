use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
};
use common::{
    constant::{PICTURE, ROOT, USER_CACHE_TTL},
    error::{ErrorMessage, HttpError},
    validation::user_dto::{UpdateUserDTO, UpdateUserPictureDTO, UserResponse},
};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, db::AccountExt};

pub fn profile_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route(ROOT, get(get_me))
        .route(ROOT, put(update_user_profile))
        .route(PICTURE, put(update_user_picture))
}

async fn get_me(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    let cache_key = format!("user:{}", user_id);

    let mut redis = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let cached: Option<String> = redis::cmd("GET")
        .arg(&cache_key)
        .query_async(&mut *redis)
        .await
        .unwrap_or(None);

    if let Some(cached_json) = cached {
        let user_response: UserResponse = serde_json::from_str(&cached_json)
            .map_err(|e| HttpError::server_error(e.to_string()))?;
        return Ok((StatusCode::OK, Json(user_response)));
    }

    let user = app_state
        .pg_client
        .get_user_by_id(user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(ErrorMessage::UserNotFound.to_string()))?;

    let user_response = UserResponse::from(user);

    if let Ok(json) = serde_json::to_string(&user_response) {
        let _: Result<(), _> = redis::cmd("SET")
            .arg(&cache_key)
            .arg(&json)
            .arg("EX")
            .arg(USER_CACHE_TTL)
            .query_async::<()>(&mut *redis)
            .await;
    }

    Ok((StatusCode::OK, Json(user_response)))
}

async fn update_user_profile(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<UpdateUserDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let updated_user = app_state
        .pg_client
        .update_user(user_id, body.name, body.email, body.mobile_no)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user_response = UserResponse::from(updated_user);

    let cache_key = format!("user:{}", user_id);
    let mut redis = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if let Ok(json) = serde_json::to_string(&user_response) {
        let _: Result<(), _> = redis::cmd("SET")
            .arg(&cache_key)
            .arg(&json)
            .arg("EX")
            .arg(USER_CACHE_TTL)
            .query_async::<()>(&mut *redis)
            .await;
    }

    Ok((StatusCode::OK, Json(user_response)))
}

async fn update_user_picture(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<UpdateUserPictureDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let updated_user = app_state
        .pg_client
        .update_user_picture(user_id, body.picture)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user_response = UserResponse::from(updated_user);

    let cache_key = format!("user:{}", user_id);
    let mut redis = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if let Ok(json) = serde_json::to_string(&user_response) {
        let _: Result<(), _> = redis::cmd("SET")
            .arg(&cache_key)
            .arg(&json)
            .arg("EX")
            .arg(USER_CACHE_TTL)
            .query_async::<()>(&mut *redis)
            .await;
    }

    Ok((StatusCode::OK, Json(user_response)))
}
