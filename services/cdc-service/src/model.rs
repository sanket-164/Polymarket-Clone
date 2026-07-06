use chrono::{DateTime, Utc};
use clickhouse::Row;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use uuid::Uuid;

// ---------------------------------------------------------------------
// Debezium/CDC envelope — used only to deserialize Kafka messages.
// This is NOT inserted into ClickHouse
// ---------------------------------------------------------------------

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
pub enum Operation {
    #[serde(rename = "c")]
    Create,
    #[serde(rename = "u")]
    Update,
}

// ---------------------------------------------------------------------
// Helper: fixed-point Decimal <-> ClickHouse Decimal64(SCALE)
// ---------------------------------------------------------------------

pub mod decimal64 {
    use rust_decimal::Decimal;
    use serde::{
        Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError,
        ser::Error as SerError,
    };
    use std::str::FromStr;

    const SCALE: u32 = 8; // must match the column's Decimal64(S)

    // Serialize: Decimal -> scaled i64, for writing into ClickHouse
    pub fn serialize<S: Serializer>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error> {
        let scaled = value
            .checked_mul(Decimal::from(10i64.pow(SCALE)))
            .ok_or_else(|| S::Error::custom("decimal overflow"))?
            .round();
        let as_i64: i64 = scaled
            .try_into()
            .map_err(|_| S::Error::custom("decimal does not fit in i64"))?;
        as_i64.serialize(serializer)
    }

    // Deserialize: Debezium sends decimals as JSON strings (decimal.handling.mode=string)
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Decimal, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Decimal::from_str(&raw).map_err(DeError::custom)
    }
}

pub mod datetime_rfc3339 {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer, de::Error as DeError};

    // Deserialize: Debezium sends timestamptz columns as RFC3339 strings
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        let raw = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&raw)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(DeError::custom)
    }

    // Serialize: delegate to clickhouse's millis encoder for the ClickHouse-bound side
    pub fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        clickhouse::serde::chrono::datetime64::millis::serialize(value, serializer)
    }
}

// ---------------------------------------------------------------------
// Enums -> ClickHouse Enum8
// ---------------------------------------------------------------------

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize, PartialEq)]
pub enum OrderSide {
    BUY = 1,
    SELL = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Serialize_repr, Deserialize, PartialEq)]
pub enum OrderStatus {
    PENDING = 1,
    PARTIAL = 2,
    FILLED = 3,
    CANCELLED = 4,
    EXPIRED = 5,
}

#[repr(u8)]
#[derive(Debug, Clone, Serialize_repr, Deserialize, PartialEq)]
pub enum TransactionType {
    DEPOSIT = 1,
    WITHDRAW = 2,
    BUY = 3,
    SELL = 4,
    REFUND = 5,
    PAYOUT = 6,
}

// ClickHouse row structs

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct OrderRow {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub user_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub market_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub outcome_id: Uuid,
    pub side: OrderSide,
    #[serde(with = "decimal64")]
    pub shares: Decimal,
    #[serde(with = "decimal64")]
    pub remaining_shares: Decimal,
    #[serde(with = "decimal64")]
    pub price: Decimal,
    pub status: OrderStatus,
    #[serde(with = "datetime_rfc3339")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "datetime_rfc3339")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct TradeRow {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub market_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub buy_order_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub sell_order_id: Uuid,
    #[serde(with = "decimal64")]
    pub shares: Decimal,
    #[serde(with = "decimal64")]
    pub price: Decimal,
    #[serde(with = "datetime_rfc3339")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct TransactionRow {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub wallet_id: Uuid,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(with = "decimal64")]
    pub amount: Decimal,
    #[serde(with = "datetime_rfc3339")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct HoldingRow {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub user_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub market_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub outcome_id: Uuid,
    #[serde(with = "decimal64")]
    pub shares: Decimal,
    #[serde(with = "decimal64")]
    pub locked_shares: Decimal,
    #[serde(with = "datetime_rfc3339")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "datetime_rfc3339")]
    pub updated_at: DateTime<Utc>,
}
