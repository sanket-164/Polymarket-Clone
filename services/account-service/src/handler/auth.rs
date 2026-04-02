use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::post,
};
use axum_extra::extract::cookie::Cookie;
use common::{
    error::{ErrorMessage, HttpError},
    util::jwt,
    validation::user_dto::{LoginUserDTO, RegisterUserDTO},
};
use serde_json::json;
use validator::Validate;

use crate::{AppState, db::AccountExt, util::hash};

pub fn auth_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route("/signup", post(signup))
        .route("/signin", post(signin))
}

pub async fn signup(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<RegisterUserDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let existing_user = app_state
        .pg_client
        .get_user_by_email(&body.email)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if existing_user.is_some() {
        Err(HttpError::conflict(ErrorMessage::EmailExist.to_string()))?
    }

    let hash_password =
        hash::generate(body.password).map_err(|e| HttpError::server_error(e.to_string()))?;

    let new_user = app_state
        .pg_client
        .create_user(&body.name, &body.email, &hash_password)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(new_user)))
}

pub async fn signin(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<LoginUserDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let existing_user = app_state
        .pg_client
        .get_user_by_email(&body.email)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = existing_user.ok_or(HttpError::unauthorized(
        ErrorMessage::WrongCredentials.to_string(),
    ))?;

    let password_matched = hash::compare(&body.password, &user.password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if password_matched {
        let jwt_token = jwt::generate_token(
            &user.id.to_string(),
            app_state.jwt_config.jwt_secret_key.as_bytes(),
            app_state.jwt_config.jwt_expiration_time,
        )
        .map_err(|e| HttpError::server_error(e.to_string()))?;

        let cookie_duration =
            time::Duration::minutes(app_state.jwt_config.jwt_expiration_time as i64 * 60);

        let cookie = Cookie::build(("token", jwt_token.clone()))
            .path("/")
            .max_age(cookie_duration)
            .http_only(true)
            .build();

        let response = (
            StatusCode::OK,
            Json(json!({
                "token": jwt_token
            })),
        );

        let mut headers = HeaderMap::new();

        headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());

        let mut response = response.into_response();
        response.headers_mut().extend(headers);

        Ok(response)
    } else {
        Err(HttpError::unauthorized(
            ErrorMessage::InvalidToken.to_string(),
        ))
    }
}
