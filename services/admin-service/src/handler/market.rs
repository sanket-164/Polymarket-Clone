use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use common::{
    constant::ROOT,
    error::HttpError,
    model::{MarketOutcomes, NatsMessage},
    nats_handler::PublishMessage,
    validation::admin_dto::CreateMarketDTO,
};
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

    let market_message = NatsMessage {
        order: None,
        market: Some(market.market.clone()),
        outcomes: Some(MarketOutcomes {
            first_outcome: first_outcome.clone(),
            second_outcome: second_outcome.clone(),
        }),
    };

    app_state
        .publisher
        .publish_message(market_message, PublishMessage::CreateMarket)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let first_outcome_order = app_state
        .pg_client
        .insert_sell_order(first_outcome)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let first_order_message = NatsMessage {
        order: Some(first_outcome_order),
        market: None,
        outcomes: None,
    };

    app_state
        .publisher
        .publish_message(first_order_message, PublishMessage::PlaceOrder)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let second_outcome_order = app_state
        .pg_client
        .insert_sell_order(second_outcome)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let second_order_message = NatsMessage {
        order: Some(second_outcome_order),
        market: None,
        outcomes: None,
    };

    app_state
        .publisher
        .publish_message(second_order_message, PublishMessage::PlaceOrder)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(market)))
}
