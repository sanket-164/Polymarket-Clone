use common::model::Trade;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::consumer::{ConsumerEvent, Operation};

pub struct TradeConsumer {
    pub consumer: StreamConsumer,
}

impl TradeConsumer {
    pub fn init() -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("group.id", "trade-rust-consumer")
            .set("auto.offset.reset", "earliest")
            .set("enable.auto.commit", "true")
            .set("auto.commit.interval.ms", "1000")
            .set("session.timeout.ms", "6000")
            .create()
            .expect("Failed to create Kafka consumer");

        TradeConsumer { consumer }
    }

    pub async fn listen(self) {
        self.consumer
            .subscribe(&["polymarket.public.trades"])
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
