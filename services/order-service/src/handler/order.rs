use std::sync::Arc;

use axum::{
    Extension, Json, Router, extract::State, http::StatusCode, response::IntoResponse,
    routing::post,
};
use chrono::Utc;
use common::{
    error::{ErrorMessage, HttpError},
    model::market::{MarketStatus, OrderType},
    validation::order_dto::PlaceOrderDTO,
};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, db::OrderExt};

pub fn order_handler() -> Router<Arc<AppState>> {
    Router::new().route("/", post(place_order))
}

async fn place_order(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(body): Json<PlaceOrderDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let market = app_state
        .pg_client
        .get_market_by_id(body.market_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(
            ErrorMessage::MarketNotFound.to_string(),
        ))?;

    if market.status != MarketStatus::ACTIVE || market.close_at < Utc::now() {
        return Err(HttpError::bad_request(
            ErrorMessage::MarketIsNotActive.to_string(),
        ));
    }

    app_state
        .pg_client
        .get_outcome_by_id(body.outcome_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(
            ErrorMessage::OutcomeNotFound.to_string(),
        ))?;

    let order;

    match body.order_type {
        OrderType::BUY => {
            let wallet = app_state
                .pg_client
                .get_user_wallet(user_id)
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;

            if wallet.balance < body.price * body.shares {
                return Err(HttpError::forbidden(
                    ErrorMessage::InsufficientBalance.to_string(),
                ));
            }

            order = app_state
                .pg_client
                .insert_order(
                    user_id,
                    body.market_id,
                    body.outcome_id,
                    body.shares,
                    body.price,
                    body.order_type,
                )
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;
        }
        OrderType::SELL => {
            let holding = app_state
                .pg_client
                .get_user_holding(user_id, body.outcome_id)
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?
                .ok_or(HttpError::forbidden(
                    ErrorMessage::InsufficientShares.to_string(),
                ))?;

            if holding.shares < body.shares {
                return Err(HttpError::forbidden(
                    ErrorMessage::InsufficientShares.to_string(),
                ));
            }

            order = app_state
                .pg_client
                .insert_order(
                    user_id,
                    body.market_id,
                    body.outcome_id,
                    body.shares,
                    body.price,
                    body.order_type,
                )
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;
        }
    }

    Ok((StatusCode::CREATED, Json(order)))
}
