use common::model::Transaction;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::consumer::{ConsumerEvent, Operation};

pub struct TransactionConsumer {
    pub consumer: StreamConsumer,
}

impl TransactionConsumer {
    pub fn init() -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("group.id", "transaction-rust-consumer")
            .set("auto.offset.reset", "earliest")
            .set("enable.auto.commit", "true")
            .set("auto.commit.interval.ms", "1000")
            .set("session.timeout.ms", "6000")
            .create()
            .expect("Failed to create Kafka consumer");

        TransactionConsumer { consumer }
    }

    pub async fn listen(self) {
        self.consumer
            .subscribe(&["polymarket.public.transactions"])
            .expect("Failed to subscribe to topic");

        println!("Transaction Consumer started, waiting for messages...");

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

                    match serde_json::from_str::<ConsumerEvent<Transaction>>(payload) {
                        Ok(event) => handle_transaction_event(event).await,
                        Err(e) => eprintln!("Failed to parse event: {} \nRaw: {}", e, payload),
                    }
                }
            }
        }
    }
}

async fn handle_transaction_event(event: ConsumerEvent<Transaction>) {
    match event.op {
        Operation::Create => {
            if let Some(after) = event.after {
                println!(
                    "NEW TRANSACTION | id={} wallet={} type={:?} amount={}",
                    after.id, after.wallet_id, after.transaction_type, after.amount
                );
                // TODO: insert into ClickHouse `transaction` table
            }
        }
        Operation::Update => {
            // transaction is insert-only (no UPDATE statements on this table),
            // so this branch should realistically never fire. Kept defensively.
            println!("TRANSACTION UPDATE (unexpected — transaction should be immutable)");
            if let Some(before) = event.before {
                println!(
                    "  before | type={:?} amount={}",
                    before.transaction_type, before.amount
                );
            }
            if let Some(after) = event.after {
                println!(
                    "  after  | type={:?} amount={}",
                    after.transaction_type, after.amount
                );
            }
        }
    }
}
