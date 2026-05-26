use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use common::{
    constant::ROOT,
    error::HttpError,
    validation::user_dto::{HoldingQueryDTO, HoldingsResponse},
};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, db::HoldingExt};

pub fn holding_handler() -> Router<Arc<AppState>> {
    Router::new().route(ROOT, get(get_user_holdings))
}

async fn get_user_holdings(
    Query(query_params): Query<HoldingQueryDTO>,
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

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

    let holdings = app_state
        .pg_client
        .get_user_holdings(user_id, order_by, limit, skip)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::OK, Json(HoldingsResponse::from(holdings))))
}
