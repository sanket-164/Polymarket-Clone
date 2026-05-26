use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use common::{
    constant::{ID, ROOT},
    error::HttpError,
    model::MarketStatus,
    validation::user_dto::MarketQueryDTO,
};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, db::MarketExt};

pub fn market_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route(ROOT, get(get_markets))
        .route(ID, get(get_market_details))
}

async fn get_markets(
    Query(query_params): Query<MarketQueryDTO>,
    State(app_state): State<Arc<AppState>>,
    Extension(_user_id): Extension<Uuid>,
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
    Extension(_user_id): Extension<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    let market = app_state
        .pg_client
        .get_market_details(id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::OK, Json(market)))
}
