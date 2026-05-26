use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use common::{
    constant::{BALANCE, DEPOSIT, TRANSACTIONS, WITHDRAW},
    error::{ErrorMessage, HttpError},
    validation::user_dto::{DepositBalanceDTO, TransactionsQueryDTO},
};
use rust_decimal::{Decimal, prelude::Zero};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, db::WalletExt};

pub fn wallet_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route(BALANCE, get(get_balance))
        .route(DEPOSIT, post(deposite_balance))
        .route(WITHDRAW, post(withdraw_balance))
        .route(TRANSACTIONS, get(get_user_transactions))
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

async fn get_user_transactions(
    Query(query_params): Query<TransactionsQueryDTO>,
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let order_by = format!(
        "{} {}",
        query_params
            .order_field
            .unwrap_or_else(|| "created_at".to_string()),
        query_params.order_by.unwrap_or_else(|| "DESC".to_string()),
    );

    let limit = match query_params.limit {
        Some(l) => l,
        _ => 10,
    };

    let skip = match query_params.skip {
        Some(s) => s,
        _ => 0,
    };

    let transactions = app_state
        .pg_client
        .get_transactions(
            user_id,
            query_params.transaction_type,
            order_by,
            limit,
            skip,
        )
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::OK, Json(transactions)))
}
