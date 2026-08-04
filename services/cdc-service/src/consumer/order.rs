use chrono::Utc;
use common::constant::{
    AUTO_COMMIT_INTERVAL_MS, AUTO_OFFSET_RESET, CDC_ORDER_TOPIC, ENABLE_AUTO_COMMIT,
    ORDER_GROUP_ID, SESSION_TIMEOUT_MS,
};
use common::model::{FeedMessage, OrderFeed};
use deadpool_redis::Pool;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rust_decimal::prelude::ToPrimitive;

use crate::model::{OrderSide, OrderStatus};
use crate::nats_handler::NatsHandler;
use crate::{
    ch_client::CHClient,
    model::{ConsumerEvent, Operation, OrderRow},
};

pub struct OrderConsumer {
    pub consumer: StreamConsumer,
    pub ch_client: CHClient,
    pub publisher: NatsHandler,
    pub redis_pool: Pool,
}

impl OrderConsumer {
    pub fn init(
        bootstrap_servers: &str,
        ch_client: CHClient,
        publisher: NatsHandler,
        redis_pool: Pool,
    ) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", ORDER_GROUP_ID)
            .set("auto.offset.reset", AUTO_OFFSET_RESET)
            .set("enable.auto.commit", ENABLE_AUTO_COMMIT)
            .set("auto.commit.interval.ms", AUTO_COMMIT_INTERVAL_MS)
            .set("session.timeout.ms", SESSION_TIMEOUT_MS)
            .create()
            .expect("Failed to create Kafka consumer");

        Self {
            consumer,
            ch_client,
            publisher,
            redis_pool,
        }
    }

    pub async fn listen(self) {
        self.consumer
            .subscribe(&[CDC_ORDER_TOPIC])
            .expect("Failed to subscribe to topic");

        println!("Order Consumer started, waiting for messages...");

        loop {
            match self.consumer.recv().await {
                Err(e) => eprintln!("Kafka error: {}", e),
                Ok(msg) => {
                    let payload = match msg.payload_view::<str>() {
                        Some(Ok(s)) => s,
                        Some(Err(e)) => {
                            eprintln!("Error deserializing message payload: {:?}", e);
                            continue;
                        }
                        None => {
                            println!("Tombstone message received (delete), skipping");
                            continue;
                        }
                    };

                    match serde_json::from_str::<ConsumerEvent<OrderRow>>(payload) {
                        Ok(event) => {
                            handle_order_event(
                                event,
                                &self.ch_client,
                                &self.publisher,
                                &self.redis_pool,
                            )
                            .await
                        }
                        Err(e) => eprintln!("Failed to parse event: {} \nRaw: {}", e, payload),
                    }
                }
            }
        }
    }
}

async fn handle_order_event(
    event: ConsumerEvent<OrderRow>,
    ch_client: &CHClient,
    publisher: &NatsHandler,
    redis_pool: &Pool,
) {
    match event.op {
        Operation::Create => {
            if let Some(after) = event.after {
                println!(
                    "NEW ORDER | id={} user={} side={:?} shares={} price={} status={:?}",
                    after.id, after.user_id, after.side, after.shares, after.price, after.status
                );

                if let Err(err) = ch_client.insert_order(&after).await {
                    eprintln!("Failed to insert order into ClickHouse: {}", err);
                }
            }
        }

        Operation::Update => {
            println!("ORDER UPDATE");

            if let Some(before) = event.before {
                println!(
                    "before | status={:?} remaining={}",
                    before.status, before.remaining_shares
                );
            }

            if let Some(after) = event.after {
                println!(
                    "after | status={:?} remaining={}",
                    after.status, after.remaining_shares
                );

                if let Err(err) = ch_client.insert_order(&after).await {
                    eprintln!("Failed to update order in ClickHouse: {}", err);
                }

                if after.status == OrderStatus::EXPIRED {
                    let mut redis = match redis_pool.get().await {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Failed to get Redis connection: {e}");
                            return;
                        }
                    };

                    let base_key = format!(
                        "orderbook:{}:{}:{}",
                        after.market_id,
                        after.outcome_id,
                        match after.side {
                            OrderSide::BUY => "buy",
                            OrderSide::SELL => "sell",
                        }
                    );

                    let qty_key = format!("{}:qty", base_key);
                    let price_str = after.price.normalize().to_string();

                    // Update the shares in HashMap
                    let new_qty: f64 = match redis::cmd("HINCRBYFLOAT")
                        .arg(&qty_key)
                        .arg(&price_str)
                        .arg(-after.remaining_shares.to_f64().unwrap_or(0.0))
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

                    if let Err(e) = redis::cmd("SET")
                        .arg(&format!("orderbook:{}:timestamp", after.market_id))
                        .arg(Utc::now().timestamp_millis())
                        .query_async::<()>(&mut *redis)
                        .await
                    {
                        eprintln!("Redis SET failed: {:?}", e);
                    }

                    let feed_message = FeedMessage::OrderFeed {
                        feed: OrderFeed {
                            market_id: after.market_id,
                            outcome_id: after.outcome_id,
                            side: match after.side {
                                OrderSide::BUY => common::model::OrderSide::BUY,
                                OrderSide::SELL => common::model::OrderSide::SELL,
                            },
                            quantity: -after.remaining_shares, // negative to signal reduction to feed subscribers
                            price: after.price.normalize(),
                            timestamp: Utc::now().timestamp_millis(),
                        },
                    };

                    if let Err(e) = publisher.feed_market_order(feed_message).await {
                        eprintln!("Failed to publish feed update OrderFeed message: {:?}", e);
                    }
                }
            }
        }
    }
}
