use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
};
use chrono::Utc;
use common::{
    constant::{ID, MARKET_CACHE_TTL, MARKET_ID, OUTCOME_ID, RESOLVE, ROOT, SNAPSHOT},
    error::{ErrorMessage, HttpError},
    model::{
        FeedMessage, MarketOutcomes, MarketStatus, MarketWithOutcomes, MatcherMessage,
        ResolveMessage,
    },
    validation::{admin_dto::CreateMarketDTO, user_dto::MarketQueryDTO},
};
use rust_decimal::prelude::ToPrimitive;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, db::MarketExt};

pub fn market_handler() -> Router<Arc<AppState>> {
    Router::new().route(ROOT, post(create_market)).route(
        &format!("{MARKET_ID}{RESOLVE}{OUTCOME_ID}"),
        put(resolve_market),
    )
}

pub fn public_market_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route(&format!("{SNAPSHOT}{ID}"), get(market_snapshot))
        .route(ROOT, get(get_markets))
        .route(ID, get(get_market_details))
}

async fn create_market(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<CreateMarketDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let market = app_state
        .pg_client
        .create_market(body)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let first_outcome = market.first_outcome.clone();
    let second_outcome = market.second_outcome.clone();

    let matcher_market_message = MatcherMessage::CreateMarket {
        market: market.market.clone(),
        outcomes: MarketOutcomes {
            first_outcome: first_outcome.clone(),
            second_outcome: second_outcome.clone(),
        },
    };

    app_state
        .publisher
        .matcher_create_market(matcher_market_message)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let feed_market_message = FeedMessage::CreateMarket {
        market_id: market.market.id,
    };

    app_state
        .publisher
        .feed_create_market(feed_market_message)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let first_outcome_order = app_state
        .pg_client
        .insert_sell_order(first_outcome)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let second_outcome_order = app_state
        .pg_client
        .insert_sell_order(second_outcome)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    // Push both initial sell orders into the aggregated Redis orderbook.
    let mut redis = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    for order in [&first_outcome_order, &second_outcome_order] {
        let base_key = format!("orderbook:{}:{}:sell", order.market_id, order.outcome_id);
        let qty_key = format!("{}:qty", base_key);
        let price_str = order.price.normalize().to_string();
        let shares_f64 = order.shares.to_f64();

        redis::pipe()
            .cmd("HINCRBYFLOAT")
            .arg(&qty_key)
            .arg(&price_str)
            .arg(shares_f64)
            .cmd("ZADD")
            .arg(&base_key)
            .arg(order.price.to_f64())
            .arg(&price_str)
            .query_async::<()>(&mut *redis)
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?;
    }

    let first_order_message = MatcherMessage::PlaceOrder {
        order: first_outcome_order,
    };

    let second_order_message = MatcherMessage::PlaceOrder {
        order: second_outcome_order,
    };

    app_state
        .publisher
        .matcher_place_order(first_order_message)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state
        .publisher
        .matcher_place_order(second_order_message)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(market)))
}

async fn resolve_market(
    State(app_state): State<Arc<AppState>>,
    Path((market_id, outcome_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, HttpError> {
    let market = app_state
        .pg_client
        .get_market_by_id(market_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(
            ErrorMessage::MarketNotFound.to_string(),
        ))?;

    if (market.status != MarketStatus::ACTIVE && market.status != MarketStatus::CLOSED)
        || market.close_at > Utc::now()
    {
        return Err(HttpError::bad_request(
            ErrorMessage::MarketIsNotClosed.to_string(),
        ));
    }

    app_state
        .pg_client
        .get_market_outcome(market_id, outcome_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(
            ErrorMessage::OutcomeNotFound.to_string(),
        ))?;

    let resolve_message = ResolveMessage::ResolveMarket {
        market_id,
        winning_outcome_id: outcome_id,
    };

    app_state
        .publisher
        .resolve_market(resolve_message)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(StatusCode::OK)
}

async fn get_markets(
    Query(query_params): Query<MarketQueryDTO>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, HttpError> {
    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let status = match query_params.status {
        Some(s) => s,
        _ => MarketStatus::ACTIVE,
    };

    let limit = match query_params.limit {
        Some(l) => l,
        _ => 10,
    };

    let skip = match query_params.skip {
        Some(s) => s,
        _ => 0,
    };

    let order_by = format!(
        "{} {}",
        query_params
            .order_field
            .unwrap_or_else(|| "created_at".to_string()),
        query_params.order_by.unwrap_or_else(|| "DESC".to_string()),
    );

    let markets = app_state
        .pg_client
        .get_markets(
            status,
            query_params.category,
            query_params.start_after,
            query_params.start_before,
            query_params.close_after,
            query_params.close_before,
            order_by,
            limit,
            skip,
        )
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::OK, Json(markets)))
}

async fn get_market_details(
    Path(id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, HttpError> {
    let cache_key = format!("market:{}", id);

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
        let market: MarketWithOutcomes = serde_json::from_str(&cached_json)
            .map_err(|e| HttpError::server_error(e.to_string()))?;
        return Ok((StatusCode::OK, Json(market)));
    }

    let market = app_state
        .pg_client
        .get_market_details(id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(
            ErrorMessage::MarketNotFound.to_string(),
        ))?;

    if let Ok(json) = serde_json::to_string(&market) {
        let _: Result<(), _> = redis::cmd("SET")
            .arg(&cache_key)
            .arg(&json)
            .arg("EX")
            .arg(MARKET_CACHE_TTL)
            .query_async(&mut *redis)
            .await;
    }

    Ok((StatusCode::OK, Json(market)))
}

async fn market_snapshot(
    Path(market_id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, HttpError> {
    let market = app_state
        .pg_client
        .get_market_details(market_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(
            ErrorMessage::MarketNotFound.to_string(),
        ))?;

    let outcomes = vec![market.first_outcome, market.second_outcome];

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
