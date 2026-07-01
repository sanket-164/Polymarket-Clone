pub mod holding;
pub mod order;
pub mod trade;
pub mod transaction;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ConsumerEvent<T> {
    pub before: Option<T>,
    pub after: Option<T>,
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
