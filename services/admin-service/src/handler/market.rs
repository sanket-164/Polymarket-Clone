use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use common::{
    constant::ROOT,
    error::HttpError,
    model::{MarketOutcomes, MatcherMessage},
    validation::admin_dto::CreateMarketDTO,
};
use rust_decimal::prelude::ToPrimitive;
use validator::Validate;

use crate::{AppState, db::MarketExt};

pub fn market_handler() -> Router<Arc<AppState>> {
    Router::new().route(ROOT, post(create_market))
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

    let market_message = MatcherMessage {
        order: None,
        market: Some(market.market.clone()),
        outcomes: Some(MarketOutcomes {
            first_outcome: first_outcome.clone(),
            second_outcome: second_outcome.clone(),
        }),
    };

    app_state
        .publisher
        .create_market(market_message)
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

    let first_order_message = MatcherMessage {
        order: Some(first_outcome_order),
        market: None,
        outcomes: None,
    };

    let second_order_message = MatcherMessage {
        order: Some(second_outcome_order),
        market: None,
        outcomes: None,
    };

    app_state
        .publisher
        .place_order(first_order_message)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state
        .publisher
        .place_order(second_order_message)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(market)))
}
