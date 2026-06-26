use common::model::Order;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::util::get_rdkafka_version;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderEvent {
    pub before: Option<Order>,
    pub after: Option<Order>,
    pub source: Source,
    pub op: Operation,
    pub ts_ms: i64,
    pub transaction: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Source {
    pub version: String,
    pub connector: String,
    pub name: String,
    pub ts_ms: i64,
    pub db: String,
    pub schema: String,
    pub table: String,
    #[serde(rename = "txId")]
    pub tx_id: Option<i64>,
    pub lsn: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    #[serde(rename = "c")]
    Create,
    #[serde(rename = "u")]
    Update,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (version_n, version_s) = get_rdkafka_version();
    println!("rdkafka version: 0x{:08x}, {}", version_n, version_s);

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", "orders-rust-consumer")
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "1000")
        .set("session.timeout.ms", "6000")
        .create()
        .expect("Failed to create Kafka consumer");

    consumer
        .subscribe(&["orders.public.orders"])
        .expect("Failed to subscribe to topic");

    println!("Consumer started, waiting for messages...");

    loop {
        match consumer.recv().await {
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

                match serde_json::from_str::<OrderEvent>(payload) {
                    Ok(event) => handle_order_event(event).await,
                    Err(e) => eprintln!("Failed to parse event: {} \nRaw: {}", e, payload),
                }
            }
        }
    }
}

async fn handle_order_event(event: OrderEvent) {
    match event.op {
        Operation::Create => {
            if let Some(after) = event.after {
                println!(
                    "NEW ORDER  | id={} user={} type={:?} shares={} price={} status={:?}",
                    after.id, after.user_id, after.side, after.shares, after.price, after.status
                );
            }
        }
        Operation::Update => {
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
            }
        }
    }
}
