use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use common::{
    constant::{ID, ROOT, SNAPSHOT},
    error::{ErrorMessage, HttpError},
    model::{FeedMessage, MarketStatus, MatcherMessage, OrderFeed, OrderType},
    validation::order_dto::{OrderQueryDTO, PlaceOrderDTO},
};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, db::OrderExt};

pub fn order_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route(ROOT, get(get_orders))
        .route(ROOT, post(place_order))
        .route(&format!("{}{}", SNAPSHOT, ID), get(market_snapshot))
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

    let market_outcome = app_state
        .pg_client
        .get_market_outcome(body.outcome_id, body.market_id)
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
                .buy_order(
                    user_id,
                    body.market_id,
                    body.outcome_id,
                    body.shares,
                    body.price,
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
                .sell_order(
                    user_id,
                    body.market_id,
                    body.outcome_id,
                    body.shares,
                    body.price,
                )
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;
        }
    }

    let order_message = MatcherMessage::PlaceOrder {
        order: order.clone(),
    };

    app_state
        .publisher
        .matcher_place_order(order_message)
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
    let price_str = body.price.normalize().to_string();
    let shares_f64 = body.shares.to_f64();
    let price_f64 = body.price.to_f64();

    redis::pipe()
        .cmd("HINCRBYFLOAT")
        .arg(format!("{}:qty", orderbook_key))
        .arg(&price_str)
        .arg(shares_f64)
        .cmd("ZADD")
        .arg(&orderbook_key)
        .arg(price_f64)
        .arg(&price_str)
        .query_async::<()>(&mut *redis)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let aggregated_qty: Option<String> = redis::cmd("HGET")
        .arg(format!("{}:qty", orderbook_key))
        .arg(&price_str)
        .query_async(&mut *redis)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let aggregated_quantity = aggregated_qty
        .and_then(|v| Decimal::from_str_exact(&v).ok())
        .unwrap_or(body.shares);

    let feed_order_message = FeedMessage::OrderFeed {
        feed: OrderFeed {
            market_id: market_outcome.market_id,
            outcome_id: market_outcome.id,
            quantity: aggregated_quantity,
            price: body.price,
        },
    };

    app_state
        .publisher
        .feed_market_order(feed_order_message)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(order)))
}

async fn market_snapshot(
    Path(market_id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
    Extension(_user_id): Extension<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    app_state
        .pg_client
        .get_market_by_id(market_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(
            ErrorMessage::MarketNotFound.to_string(),
        ))?;

    let outcomes = app_state
        .pg_client
        .get_market_outcomes(market_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let mut redis = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let mut snapshot = vec![];

    for outcome in &outcomes {
        let buy_key = format!("orderbook:{}:{}:buy", market_id, outcome.id);
        let sell_key = format!("orderbook:{}:{}:sell", market_id, outcome.id);
        let buy_qty_key = format!("{}:qty", buy_key);
        let sell_qty_key = format!("{}:qty", sell_key);

        // fetch the top 10 price levels from both sorted sets.
        let (top_buys, top_sells): (Vec<String>, Vec<String>) = redis::pipe()
            .cmd("ZREVRANGE")
            .arg(&buy_key)
            .arg(0)
            .arg(9)
            .cmd("ZRANGE")
            .arg(&sell_key)
            .arg(0)
            .arg(9)
            .query_async(&mut *redis)
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?;

        // Only the qty fields we actually need.
        let buy_levels = if top_buys.is_empty() {
            vec![]
        } else {
            let buy_qtys: Vec<Option<f64>> = redis::cmd("HMGET")
                .arg(&buy_qty_key)
                .arg(top_buys.as_slice())
                .query_async(&mut *redis)
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;

            top_buys
                .into_iter()
                .zip(buy_qtys)
                .map(|(price, qty)| json!({ "price": price, "qty": qty.unwrap_or(0.0) }))
                .collect()
        };

        let sell_levels = if top_sells.is_empty() {
            vec![]
        } else {
            let sell_qtys: Vec<Option<f64>> = redis::cmd("HMGET")
                .arg(&sell_qty_key)
                .arg(top_sells.as_slice())
                .query_async(&mut *redis)
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;

            top_sells
                .into_iter()
                .zip(sell_qtys)
                .map(|(price, qty)| json!({ "price": price, "qty": qty.unwrap_or(0.0) }))
                .collect()
        };

        snapshot.push(json!({
            "outcome_id":    outcome.id,
            "outcome_label": outcome.label,
            "buy":           buy_levels,
            "sell":          sell_levels,
        }));
    }

    Ok((StatusCode::OK, Json(snapshot)))
}
