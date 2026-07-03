use common::constant::{
    AUTO_COMMIT_INTERVAL_MS, AUTO_OFFSET_RESET, CDC_ORDER_TOPIC, ENABLE_AUTO_COMMIT,
    ORDER_GROUP_ID, SESSION_TIMEOUT_MS,
};
use common::model::Order;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::dto::{ConsumerEvent, Operation};

pub struct OrderConsumer {
    pub consumer: StreamConsumer,
}

impl OrderConsumer {
    pub fn init(bootstrap_servers: &str) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", ORDER_GROUP_ID)
            .set("auto.offset.reset", AUTO_OFFSET_RESET)
            .set("enable.auto.commit", ENABLE_AUTO_COMMIT)
            .set("auto.commit.interval.ms", AUTO_COMMIT_INTERVAL_MS)
            .set("session.timeout.ms", SESSION_TIMEOUT_MS)
            .create()
            .expect("Failed to create Kafka consumer");

        OrderConsumer { consumer }
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

                    match serde_json::from_str::<ConsumerEvent<Order>>(payload) {
                        Ok(event) => handle_order_event(event).await,
                        Err(e) => eprintln!("Failed to parse event: {} \nRaw: {}", e, payload),
                    }
                }
            }
        }
    }
}

async fn handle_order_event(event: ConsumerEvent<Order>) {
    match event.op {
        Operation::Create => {
            if let Some(after) = event.after {
                println!(
                    "NEW ORDER  | id={} user={} side={:?} shares={} price={} status={:?}",
                    after.id, after.user_id, after.side, after.shares, after.price, after.status
                );
                // TODO: insert into ClickHouse `order` table
            }
        }
        Operation::Update => {
            // order IS mutable (status/remaining_shares change as fills happen),
            // so unlike Trades/Transactions, this branch will fire regularly.
            println!("ORDER UPDATE");
            if let Some(before) = event.before {
                println!(
                    "  before | status={:?} remaining={}",
                    before.status, before.remaining_shares
                );
            }
            if let Some(after) = event.after {
                println!(
                    "  after  | status={:?} remaining={}",
                    after.status, after.remaining_shares
                );
                // TODO: insert into ClickHouse (ReplacingMergeTree(updated_at) handles dedup on merge)
            }
        }
    }
}
