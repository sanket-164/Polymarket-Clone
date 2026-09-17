use crate::{db::OrderExt, nats_handler::NatsHandler};
use common::{
    constant::TRADE_STREAM,
    database::client::PGClient,
    model::{FeedMessage, OrderFeed, OrderSide, TradeMessage},
};
use deadpool_redis::Pool;
use futures::StreamExt;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use std::sync::Arc;
use uuid::Uuid;

async fn update_orderbook(
    market_id: Uuid,
    outcome_id: Uuid,
    side: OrderSide,
    price: Decimal,
    quantity: Decimal,
    trade: Option<Decimal>,
    timestamp: i64,
    redis: &mut deadpool_redis::Connection,
    nats_handler: &NatsHandler,
) {
    let market_id_str = market_id.to_string();
    let outcome_id_str = outcome_id.to_string();

    let base_key = format!(
        "orderbook:{}:{}:{}",
        market_id_str,
        outcome_id_str,
        match side {
            OrderSide::BUY => "buy",
            OrderSide::SELL => "sell",
        }
    );
    let qty_key = format!("{}:qty", base_key);
    let price_str = price.normalize().to_string();

    // Update the shares in HashMap
    let new_qty: f64 = match redis::cmd("HINCRBYFLOAT")
        .arg(&qty_key)
        .arg(&price_str)
        .arg(-quantity.to_f64().unwrap_or(0.0))
        .query_async(&mut *redis)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Redis HINCRBYFLOAT failed: {:?}", e);
            return;
        }
    };

    // Remove price & shares from HashMap & SortedSet if share's quantity is 0
    if new_qty <= 0.0 {
        if let Err(e) = redis::pipe()
            .cmd("HDEL")
            .arg(&qty_key)
            .arg(&price_str)
            .cmd("ZREM")
            .arg(&base_key)
            .arg(&price_str)
            .query_async::<()>(&mut *redis)
            .await
        {
            eprintln!("Redis cleanup failed: {:?}", e);
        }
    }

    let feed_message = FeedMessage::OrderFeed {
        feed: OrderFeed {
            market_id,
            outcome_id,
            side,
            quantity: -quantity,
            price: price.normalize(),
            trade: trade,
            timestamp,
        },
    };

    if let Err(e) = nats_handler.feed_market_order(feed_message).await {
        eprintln!("Failed to publish feed update OrderFeed message: {:?}", e);
    }
}

pub async fn start_consumer(
    nats_handler: Arc<NatsHandler>,
    pg_client: Arc<PGClient>,
    redis_pool: Arc<Pool>,
) {
    let mut message_stream = nats_handler
        .get_message_stream(TRADE_STREAM)
        .await
        .expect("Failed to get messages");

    let mut redis = match redis_pool.get().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to get Redis connection: {e}");
            return;
        }
    };

    println!("Trade Consumer is ready to receive message");

    while let Some(msg) = message_stream.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Message error: {e}");
                continue;
            }
        };

        let message: TradeMessage = match serde_json::from_slice(&msg.payload) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Deserialize error: {e}");
                let _ = msg.ack().await;
                continue;
            }
        };

        match message {
            TradeMessage::UpdateOrders {
                buy,
                sell,
                timestamp,
            } => {
                let trade = match pg_client.trade(buy.clone(), sell.clone()).await {
                    Ok(trade) => trade,
                    Err(e) => {
                        eprintln!("Trade error: {e}");
                        let _ = msg.ack().await;
                        continue;
                    }
                };

                let filled_shares = buy.remaining_shares.min(sell.remaining_shares);

                for order in [buy, sell] {
                    update_orderbook(
                        order.market_id,
                        order.outcome_id,
                        order.side,
                        order.price,
                        filled_shares,
                        Some(trade.price),
                        timestamp,
                        &mut redis,
                        &nats_handler,
                    )
                    .await;
                }

                if let Err(e) = redis::cmd("SET")
                    .arg(&format!("orderbook:{}:timestamp", trade.market_id))
                    .arg(timestamp)
                    .query_async::<()>(&mut *redis)
                    .await
                {
                    eprintln!("Redis SET failed: {:?}", e);
                }
            }

            TradeMessage::CancelOrder { order, timestamp } => {
                let cancel_order = match pg_client.cancel_order(order.clone()).await {
                    Ok(order) => order,
                    Err(e) => {
                        eprintln!("Cancel order error: {e}");
                        let _ = msg.ack().await;
                        continue;
                    }
                };

                update_orderbook(
                    cancel_order.market_id,
                    cancel_order.outcome_id,
                    cancel_order.side,
                    cancel_order.price,
                    cancel_order.remaining_shares,
                    None,
                    timestamp,
                    &mut redis,
                    &nats_handler,
                )
                .await;
            }
        }

        if let Err(e) = msg.ack().await {
            eprintln!("Ack failed: {e}");
        }
    }
}
