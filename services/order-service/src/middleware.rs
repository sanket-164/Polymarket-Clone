use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use common::{
    constant::USER_TOKEN,
    error::{ErrorMessage, HttpError},
    util::jwt,
};

use crate::{AppState, db::AccountExt};

pub async fn auth_middleware(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, HttpError> {
    let auth_token = cookie_jar
        .get(USER_TOKEN)
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });

    let token = auth_token
        .ok_or_else(|| HttpError::server_error(ErrorMessage::TokenNotGiven.to_string()))?;

    let token_details =
        match jwt::decode_token(token, app_state.jwt_config.jwt_secret_key.as_bytes()) {
            Ok(token_details) => token_details,
            Err(_) => {
                return Err(HttpError::unauthorized(
                    ErrorMessage::InvalidToken.to_string(),
                ));
            }
        };

    let user_id = uuid::Uuid::parse_str(&token_details)
        .map_err(|_| HttpError::unauthorized(ErrorMessage::InvalidToken.to_string()))?;

    let user = app_state
        .pg_client
        .get_user_by_id(user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if user.is_none() {
        return Err(HttpError::server_error(
            ErrorMessage::InvalidToken.to_string(),
        ));
    }

    req.extensions_mut().insert(user_id);

    Ok(next.run(req).await)
}
