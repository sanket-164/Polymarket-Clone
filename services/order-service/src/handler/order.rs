use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use common::{
    constant::{ORDER_GET, ORDER_PLACE},
    error::{ErrorMessage, HttpError},
    model::market::{MarketStatus, NatsMessage, OrderType},
    validation::order_dto::{OrderQueryDTO, PlaceOrderDTO},
};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, db::OrderExt};

pub fn order_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route(ORDER_GET, get(get_orders))
        .route(ORDER_PLACE, post(place_order))
}

async fn get_orders(
    Query(query_params): Query<OrderQueryDTO>,
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
        query_params.order_by.unwrap_or_else(|| "ASC".to_string()),
    );

    let limit = match query_params.limit {
        Some(l) => l,
        _ => 10,
    };

    let skip = match query_params.skip {
        Some(s) => s,
        _ => 0,
    };

    let orders = app_state
        .pg_client
        .get_user_orders(
            user_id,
            query_params.market_id,
            query_params.order_type,
            query_params.status,
            query_params.before,
            query_params.after,
            order_by,
            limit,
            skip,
        )
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::OK, Json(orders)))
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

    if market.start_at > Utc::now()
        || (market.status != MarketStatus::ACTIVE && market.status != MarketStatus::PENDING)
        || market.close_at < Utc::now()
    {
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

    let order_message = NatsMessage {
        order: Some(order.clone()),
        market: None,
        outcomes: None,
    };

    app_state
        .publisher
        .publish_message(
            order_message,
            common::nats_handler::PublishMessage::InsertOrder,
        )
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(order)))
}
