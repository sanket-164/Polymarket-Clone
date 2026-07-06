use common::constant::{
    AUTO_COMMIT_INTERVAL_MS, AUTO_OFFSET_RESET, CDC_HOLDING_TOPIC, ENABLE_AUTO_COMMIT,
    HOLDING_GROUP_ID, SESSION_TIMEOUT_MS,
};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::{
    ch_client::CHClient,
    model::{ConsumerEvent, HoldingRow, Operation},
};

pub struct HoldingConsumer {
    pub consumer: StreamConsumer,
    pub ch_client: CHClient,
}

impl HoldingConsumer {
    pub fn init(bootstrap_servers: &str, ch_client: CHClient) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", HOLDING_GROUP_ID)
            .set("auto.offset.reset", AUTO_OFFSET_RESET)
            .set("enable.auto.commit", ENABLE_AUTO_COMMIT)
            .set("auto.commit.interval.ms", AUTO_COMMIT_INTERVAL_MS)
            .set("session.timeout.ms", SESSION_TIMEOUT_MS)
            .create()
            .expect("Failed to create Kafka consumer");

        HoldingConsumer {
            consumer,
            ch_client,
        }
    }

    pub async fn listen(self) {
        self.consumer
            .subscribe(&[CDC_HOLDING_TOPIC])
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

                    match serde_json::from_str::<ConsumerEvent<HoldingRow>>(payload) {
                        Ok(event) => handle_holding_event(event, &self.ch_client).await,
                        Err(e) => eprintln!("Failed to parse event: {} \nRaw: {}", e, payload),
                    }
                }
            }
        }
    }
}

async fn handle_holding_event(event: ConsumerEvent<HoldingRow>, ch_client: &CHClient) {
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

                if let Err(err) = ch_client.insert_holding(&after).await {
                    eprintln!("Failed to insert holding into ClickHouse: {}", err);
                }
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

                if let Err(err) = ch_client.insert_holding(&after).await {
                    eprintln!("Failed to update holding in ClickHouse: {}", err);
                }
            }
        }
    }
}
