use common::constant::{
    AUTO_COMMIT_INTERVAL_MS, AUTO_OFFSET_RESET, CDC_TRANSACTION_TOPIC, ENABLE_AUTO_COMMIT,
    SESSION_TIMEOUT_MS, TRANSACTION_GROUP_ID,
};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::{
    ch_client::CHClient,
    model::{ConsumerEvent, Operation, TransactionRow},
};

pub struct TransactionConsumer {
    pub consumer: StreamConsumer,
    pub ch_client: CHClient,
}

impl TransactionConsumer {
    pub fn init(bootstrap_servers: &str, ch_client: CHClient) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", TRANSACTION_GROUP_ID)
            .set("auto.offset.reset", AUTO_OFFSET_RESET)
            .set("enable.auto.commit", ENABLE_AUTO_COMMIT)
            .set("auto.commit.interval.ms", AUTO_COMMIT_INTERVAL_MS)
            .set("session.timeout.ms", SESSION_TIMEOUT_MS)
            .create()
            .expect("Failed to create Kafka consumer");

        TransactionConsumer {
            consumer,
            ch_client,
        }
    }

    pub async fn listen(self) {
        self.consumer
            .subscribe(&[CDC_TRANSACTION_TOPIC])
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

                    match serde_json::from_str::<ConsumerEvent<TransactionRow>>(payload) {
                        Ok(event) => handle_transaction_event(event, &self.ch_client).await,
                        Err(e) => eprintln!("Failed to parse event: {} \nRaw: {}", e, payload),
                    }
                }
            }
        }
    }
}

async fn handle_transaction_event(event: ConsumerEvent<TransactionRow>, ch_client: &CHClient) {
    match event.op {
        Operation::Create => {
            if let Some(after) = event.after {
                println!(
                    "NEW TRANSACTION | id={} wallet={} type={:?} amount={}",
                    after.id, after.wallet_id, after.transaction_type, after.amount
                );

                if let Err(err) = ch_client.insert_transaction(&after).await {
                    eprintln!("Failed to insert transaction into ClickHouse: {}", err);
                }
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
