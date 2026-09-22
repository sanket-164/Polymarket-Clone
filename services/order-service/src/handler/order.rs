use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
};
use chrono::Utc;
use common::{
    constant::{CANCEL, DEFAULT_LIMIT, ORDER_ID, ROOT},
    error::{ErrorMessage, HttpError},
    model::{FeedMessage, MarketStatus, MatcherMessage, OrderFeed, OrderSide, OrderStatus},
};
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    db::{HoldingExt, MarketExt, OrderExt, WalletExt},
    dto::{OrderQueryDTO, PlaceOrderDTO},
};

pub fn order_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route(ROOT, get(get_orders))
        .route(ROOT, post(place_order))
        .route(&format!("{CANCEL}{ORDER_ID}"), put(cancel_order))
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
        _ => DEFAULT_LIMIT,
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
            query_params.side,
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

    match body {
        PlaceOrderDTO::Limit(limit_order) => {
            let market = app_state
                .pg_client
                .get_market_by_id(limit_order.market_id)
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

            if limit_order.expires_at > market.close_at {
                return Err(HttpError::bad_request(
                    ErrorMessage::ExceedCloseTime.to_string(),
                ));
            }

            let market_outcome = app_state
                .pg_client
                .get_market_outcome(limit_order.outcome_id, limit_order.market_id)
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?
                .ok_or(HttpError::not_found(
                    ErrorMessage::OutcomeNotFound.to_string(),
                ))?;

            if limit_order.shares > market_outcome.total_shares {
                return Err(HttpError::bad_request(
                    ErrorMessage::ExceedAvailableShares.to_string(),
                ));
            }

            let order;

            match limit_order.side {
                OrderSide::BUY => {
                    let wallet = app_state
                        .pg_client
                        .get_user_wallet(user_id)
                        .await
                        .map_err(|e| HttpError::server_error(e.to_string()))?;

                    if wallet.balance < limit_order.price * limit_order.shares {
                        return Err(HttpError::forbidden(
                            ErrorMessage::InsufficientBalance.to_string(),
                        ));
                    }

                    order = app_state
                        .pg_client
                        .buy_limit_order(
                            user_id,
                            limit_order.market_id,
                            limit_order.outcome_id,
                            limit_order.shares,
                            limit_order.price,
                            limit_order.expires_at,
                        )
                        .await
                        .map_err(|e| HttpError::server_error(e.to_string()))?;
                }
                OrderSide::SELL => {
                    let holding = app_state
                        .pg_client
                        .get_user_holding(user_id, limit_order.outcome_id)
                        .await
                        .map_err(|e| HttpError::server_error(e.to_string()))?
                        .ok_or(HttpError::forbidden(
                            ErrorMessage::InsufficientShares.to_string(),
                        ))?;

                    if holding.shares < limit_order.shares {
                        return Err(HttpError::forbidden(
                            ErrorMessage::InsufficientShares.to_string(),
                        ));
                    }

                    order = app_state
                        .pg_client
                        .sell_limit_order(
                            user_id,
                            limit_order.market_id,
                            limit_order.outcome_id,
                            limit_order.shares,
                            limit_order.price,
                            limit_order.expires_at,
                        )
                        .await
                        .map_err(|e| HttpError::server_error(e.to_string()))?;
                }
            }

            let mut redis = app_state
                .redis_pool
                .get()
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;

            let orderbook_key = format!(
                "orderbook:{}:{}:{}",
                limit_order.market_id,
                limit_order.outcome_id,
                match limit_order.side {
                    OrderSide::BUY => "buy",
                    OrderSide::SELL => "sell",
                }
            );
            let price_str = limit_order.price.normalize().to_string();
            let shares_f64 = limit_order.shares.to_f64();
            let price_f64 = limit_order.price.to_f64();

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

            let order_timestamp = order.created_at.timestamp_millis();

            redis::cmd("SET")
                .arg(&format!("orderbook:{}:timestamp", limit_order.market_id))
                .arg(order_timestamp)
                .query_async::<()>(&mut *redis)
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;

            let feed_order_message = FeedMessage::OrderFeed {
                feed: OrderFeed {
                    market_id: market_outcome.market_id,
                    outcome_id: market_outcome.id,
                    side: limit_order.side,
                    quantity: limit_order.shares,
                    price: limit_order.price.normalize(),
                    trade: None,
                    timestamp: order_timestamp,
                },
            };

            app_state
                .publisher
                .feed_market_order(feed_order_message)
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;

            app_state
                .publisher
                .matcher_limit_order(MatcherMessage::PlaceOrder {
                    order: order.clone(),
                })
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;

            Ok((StatusCode::CREATED, Json(order)))
        }

        PlaceOrderDTO::Market(market_order) => {
            let market = app_state
                .pg_client
                .get_market_by_id(market_order.market_id)
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
                .get_market_outcome(market_order.outcome_id, market_order.market_id)
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?
                .ok_or(HttpError::not_found(
                    ErrorMessage::OutcomeNotFound.to_string(),
                ))?;

            let order;

            match market_order.side {
                OrderSide::BUY => {
                    let wallet = app_state
                        .pg_client
                        .get_user_wallet(user_id)
                        .await
                        .map_err(|e| HttpError::server_error(e.to_string()))?;

                    let quote_amount = market_order.quote_amount.ok_or(HttpError::bad_request(
                        ErrorMessage::QuoteAmountNotGiven.to_string(),
                    ))?;

                    if wallet.balance < quote_amount {
                        return Err(HttpError::forbidden(
                            ErrorMessage::InsufficientBalance.to_string(),
                        ));
                    }

                    order = app_state
                        .pg_client
                        .buy_market_order(
                            user_id,
                            market_order.market_id,
                            market_order.outcome_id,
                            quote_amount,
                            market.close_at,
                        )
                        .await
                        .map_err(|e| HttpError::server_error(e.to_string()))?;
                }
                OrderSide::SELL => {
                    let holding = app_state
                        .pg_client
                        .get_user_holding(user_id, market_order.outcome_id)
                        .await
                        .map_err(|e| HttpError::server_error(e.to_string()))?
                        .ok_or(HttpError::forbidden(
                            ErrorMessage::InsufficientShares.to_string(),
                        ))?;

                    let shares = market_order.shares.ok_or(HttpError::bad_request(
                        ErrorMessage::SharesNotGiven.to_string(),
                    ))?;

                    if holding.shares < shares {
                        return Err(HttpError::forbidden(
                            ErrorMessage::InsufficientShares.to_string(),
                        ));
                    }

                    order = app_state
                        .pg_client
                        .sell_market_order(
                            user_id,
                            market_order.market_id,
                            market_order.outcome_id,
                            shares,
                            market.close_at,
                        )
                        .await
                        .map_err(|e| HttpError::server_error(e.to_string()))?;
                }
            }

            app_state
                .publisher
                .matcher_market_order(MatcherMessage::PlaceOrder {
                    order: order.clone(),
                })
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;

            Ok((StatusCode::CREATED, Json(order)))
        }
    }
}

async fn cancel_order(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(order_id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    let order = app_state
        .pg_client
        .get_order_by_id(user_id, order_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found(
            ErrorMessage::OrderNotFound.to_string(),
        ))?;

    if !(order.status == OrderStatus::PENDING || order.status == OrderStatus::PARTIAL)
        || order.expires_at <= Utc::now()
    {
        return Err(HttpError::bad_request(
            ErrorMessage::OrderNotOpen.to_string(),
        ));
    }

    app_state
        .publisher
        .matcher_cancelled_order(MatcherMessage::CancelledOrder { order })
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(StatusCode::OK)
}
