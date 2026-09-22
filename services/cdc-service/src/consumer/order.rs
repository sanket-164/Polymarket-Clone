use common::constant::{
    AUTO_COMMIT_INTERVAL_MS, AUTO_OFFSET_RESET, CDC_ORDER_TOPIC, ENABLE_AUTO_COMMIT,
    ORDER_GROUP_ID, SESSION_TIMEOUT_MS,
};
use common::model::{Order, TradeMessage};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::model::{OrderSide, OrderStatus, OrderType};
use crate::nats_handler::NatsHandler;
use crate::{
    ch_client::CHClient,
    model::{ConsumerEvent, Operation, OrderRow},
};

pub struct OrderConsumer {
    pub consumer: StreamConsumer,
    pub ch_client: CHClient,
    pub publisher: NatsHandler,
}

impl OrderConsumer {
    pub async fn init(bootstrap_servers: &str, ch_client: CHClient, nats_url: &str) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", ORDER_GROUP_ID)
            .set("auto.offset.reset", AUTO_OFFSET_RESET)
            .set("enable.auto.commit", ENABLE_AUTO_COMMIT)
            .set("auto.commit.interval.ms", AUTO_COMMIT_INTERVAL_MS)
            .set("session.timeout.ms", SESSION_TIMEOUT_MS)
            .create()
            .expect("Failed to create Kafka consumer");

        let publisher = match NatsHandler::new(nats_url).await {
            Ok(p) => p,
            Err(e) => {
                println!("Failed to connect publisher: {e}");
                std::process::exit(1);
            }
        };

        Self {
            consumer,
            ch_client,
            publisher,
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
                            handle_order_event(event, &self.ch_client, &self.publisher).await
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
) {
    match event.op {
        Operation::Create => {
            if let Some(after) = event.after {
                println!(
                    "NEW ORDER | id={} user={} side={:?} shares={} price={} status={:?} type={:?}",
                    after.id,
                    after.user_id,
                    after.side,
                    after.shares,
                    after.price,
                    after.status,
                    after.order_type
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

                if after.status == OrderStatus::EXPIRED && after.order_type == OrderType::LIMIT {
                    let expired_order_message = TradeMessage::ExpiredOrder {
                        order: Order {
                            id: after.id,
                            market_id: after.market_id,
                            outcome_id: after.outcome_id,
                            user_id: after.user_id,
                            side: match after.side {
                                OrderSide::BUY => common::model::OrderSide::BUY,
                                OrderSide::SELL => common::model::OrderSide::SELL,
                            },
                            price: after.price,
                            shares: after.shares,
                            remaining_shares: after.remaining_shares,
                            status: common::model::OrderStatus::EXPIRED,
                            average_price: after.average_price,
                            quote_amount: after.quote_amount,
                            remaining_quote: after.remaining_quote,
                            order_type: common::model::OrderType::LIMIT,
                            expires_at: after.expires_at,
                            created_at: after.created_at,
                            updated_at: after.updated_at,
                        },
                    };

                    if let Err(e) = publisher.matcher_expired_order(expired_order_message).await {
                        eprintln!("Failed to publish feed update OrderFeed message: {:?}", e);
                    }
                }
            }
        }
    }
}
