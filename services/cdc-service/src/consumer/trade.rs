use common::constant::{
    AUTO_COMMIT_INTERVAL_MS, AUTO_OFFSET_RESET, CDC_TRADE_TOPIC, ENABLE_AUTO_COMMIT,
    SESSION_TIMEOUT_MS, TRADE_GROUP_ID,
};
use common::model::Trade;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::dto::{ConsumerEvent, Operation};

pub struct TradeConsumer {
    pub consumer: StreamConsumer,
}

impl TradeConsumer {
    pub fn init(bootstrap_servers: &str) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", TRADE_GROUP_ID)
            .set("auto.offset.reset", AUTO_OFFSET_RESET)
            .set("enable.auto.commit", ENABLE_AUTO_COMMIT)
            .set("auto.commit.interval.ms", AUTO_COMMIT_INTERVAL_MS)
            .set("session.timeout.ms", SESSION_TIMEOUT_MS)
            .create()
            .expect("Failed to create Kafka consumer");

        TradeConsumer { consumer }
    }

    pub async fn listen(self) {
        self.consumer
            .subscribe(&[CDC_TRADE_TOPIC])
            .expect("Failed to subscribe to topic");

        println!("Trade Consumer started, waiting for messages...");

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

                    match serde_json::from_str::<ConsumerEvent<Trade>>(payload) {
                        Ok(event) => handle_trade_event(event).await,
                        Err(e) => eprintln!("Failed to parse event: {} \nRaw: {}", e, payload),
                    }
                }
            }
        }
    }
}

async fn handle_trade_event(event: ConsumerEvent<Trade>) {
    match event.op {
        Operation::Create => {
            if let Some(after) = event.after {
                println!(
                    "NEW TRADE | id={} market={} buy_order={} sell_order={} shares={} price={}",
                    after.id,
                    after.market_id,
                    after.buy_order_id,
                    after.sell_order_id,
                    after.shares,
                    after.price
                );
                // TODO: insert into ClickHouse `trade` table
            }
        }
        Operation::Update => {
            // trade is insert-only (no UPDATE statements on this table),
            // so this branch should realistically never fire. Kept here defensively
            println!("TRADE UPDATE (unexpected — trade should be immutable)");
            if let Some(before) = event.before {
                println!("  before | shares={} price={}", before.shares, before.price);
            }
            if let Some(after) = event.after {
                println!("  after  | shares={} price={}", after.shares, after.price);
            }
        }
    }
}
