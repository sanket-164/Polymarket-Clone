use crate::db::OrderExt;
use common::{
    constant::TRADE_STREAM,
    database::client::PGClient,
    model::{OrderSide, TradeMessage},
    nats_handler::NatsHandler,
};
use deadpool_redis::Pool;
use futures::StreamExt;
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;

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
            TradeMessage::UpdateOrders { buy, sell } => {
                if let Err(e) = pg_client.trade(buy.clone(), sell.clone()).await {
                    eprintln!("Trade error: {e}");
                    let _ = msg.ack().await;
                    continue;
                }

                let filled_shares = buy.remaining_shares.min(sell.remaining_shares);

                for order in [buy, sell] {
                    let base_key = format!(
                        "orderbook:{}:{}:{}",
                        order.market_id,
                        order.outcome_id,
                        match order.side {
                            OrderSide::BUY => "buy",
                            OrderSide::SELL => "sell",
                        }
                    );
                    let qty_key = format!("{}:qty", base_key);
                    let price_str = order.price.normalize().to_string();

                    // Update the shares in HashMap
                    let new_qty: f64 = match redis::cmd("HINCRBYFLOAT")
                        .arg(&qty_key)
                        .arg(&price_str)
                        .arg(-filled_shares.to_f64().unwrap_or(0.0))
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
                }
            }
        }

        if let Err(e) = msg.ack().await {
            eprintln!("Ack failed: {e}");
        }
    }
}
