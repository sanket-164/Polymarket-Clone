use common::model::Holding;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::consumer::{ConsumerEvent, Operation};

pub struct HoldingConsumer {
    pub consumer: StreamConsumer,
}

impl HoldingConsumer {
    pub fn init() -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("group.id", "holding-rust-consumer")
            .set("auto.offset.reset", "earliest")
            .set("enable.auto.commit", "true")
            .set("auto.commit.interval.ms", "1000")
            .set("session.timeout.ms", "6000")
            .create()
            .expect("Failed to create Kafka consumer");

        HoldingConsumer { consumer }
    }

    pub async fn listen(self) {
        self.consumer
            .subscribe(&["polymarket.public.holdings"])
            .expect("Failed to subscribe to topic");

        println!("Holding Consumer started, waiting for messages...");

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

                    match serde_json::from_str::<ConsumerEvent<Holding>>(payload) {
                        Ok(event) => handle_holding_event(event).await,
                        Err(e) => eprintln!("Failed to parse event: {} \nRaw: {}", e, payload),
                    }
                }
            }
        }
    }
}

async fn handle_holding_event(event: ConsumerEvent<Holding>) {
    match event.op {
        Operation::Create => {
            if let Some(after) = event.after {
                println!(
                    "NEW HOLDING | id={} user={} market={} outcome={} shares={} locked_shares={}",
                    after.id,
                    after.user_id,
                    after.market_id,
                    after.outcome_id,
                    after.shares,
                    after.locked_shares
                );
                // TODO: insert into ClickHouse `holding` table
            }
        }
        Operation::Update => {
            // holding IS mutable (shares/locked_shares change as fills happen),
            // so unlike Trades/Transactions, this branch will fire regularly.
            println!("HOLDING UPDATE");
            if let Some(before) = event.before {
                println!(
                    "  before | shares={} locked_shares={}",
                    before.shares, before.locked_shares
                );
            }
            if let Some(after) = event.after {
                println!(
                    "  after  | shares={} locked_shares={}",
                    after.shares, after.locked_shares
                );
                // TODO: upsert into ClickHouse (ReplacingMergeTree handles this on insert)
            }
        }
    }
}
