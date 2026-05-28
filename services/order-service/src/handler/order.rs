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
    constant::ROOT,
    error::{ErrorMessage, HttpError},
    model::{MarketStatus, NatsMessage, OrderType},
    validation::order_dto::{OrderQueryDTO, PlaceOrderDTO},
};
use redis::AsyncCommands;
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, db::OrderExt};

pub fn order_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route(ROOT, get(get_orders))
        .route(ROOT, post(place_order))
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
        .get_market_outcome(body.outcome_id, body.market_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(
            ErrorMessage::OutcomeNotFound.to_string(),
        ))?;

    let order;
    let order_type = body.order_type.clone();

    match order_type {
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
                    order_type,
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
                    order_type,
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
        .place_order(order_message)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let mut redis = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let orderbook_key = format!(
        "orderbook:{}:{}:{}",
        body.market_id,
        body.outcome_id,
        match body.order_type {
            OrderType::BUY => "buy",
            OrderType::SELL => "sell",
        }
    );

    redis
        .hincr::<_, _, _, ()>(
            format!("{}:qty", orderbook_key),
            body.price.to_string(),
            body.shares.to_f64(),
        )
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    redis::cmd("ZADD")
        .arg(orderbook_key)
        .arg(body.price.to_f64())
        .arg(body.price.to_string())
        .query_async::<()>(&mut redis)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(order)))
}
