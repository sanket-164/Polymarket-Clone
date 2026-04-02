use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use common::{
    error::{ErrorMessage, HttpError},
    validation::user_dto::DepositBalanceDTO,
};
use rust_decimal::{Decimal, prelude::Zero};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, db::AccountExt};

pub fn wallet_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_balance))
        .route("/deposit", post(deposite_balance))
        .route("/withdraw", post(withdraw_balance))
}

async fn get_balance(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    let wallet = app_state
        .pg_client
        .get_balance(user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::OK, Json(wallet)))
}

async fn deposite_balance(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<DepositBalanceDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let wallet = app_state
        .pg_client
        .deposite_balance(user_id, body.balance)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::OK, Json(wallet)))
}

async fn withdraw_balance(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<DepositBalanceDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let existing_wallet = app_state
        .pg_client
        .get_balance(user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if existing_wallet.balance - body.balance < Decimal::zero() {
        Err(HttpError::forbidden(
            ErrorMessage::InsufficientBalance.to_string(),
        ))?;
    }

    let wallet = app_state
        .pg_client
        .withdraw_balance(user_id, body.balance)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::OK, Json(wallet)))
}
