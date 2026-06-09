use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct Admin {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "market_status")]
pub enum MarketStatus {
    PENDING,
    ACTIVE,
    CLOSED,
    RESOLVED,
    CANCELLED,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow, sqlx::Type)]
pub struct Market {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: String,
    pub start_at: DateTime<Utc>,
    pub close_at: DateTime<Utc>,
    pub status: MarketStatus,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, sqlx::Type)]
pub struct Outcome {
    pub id: Uuid,
    pub market_id: Uuid,
    pub label: String,
    pub start_price: Decimal,
    pub current_price: Decimal,
    pub total_shares: Decimal,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketWithOutcomes {
    // when serialized to JSON, market fields appear at the top level
    #[serde(flatten)]
    pub market: Market,
    pub first_outcome: Outcome,
    pub second_outcome: Outcome,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, sqlx::Type, Eq)]
#[sqlx(type_name = "order_type")]
pub enum OrderType {
    BUY,
    SELL,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, sqlx::Type, Eq)]
#[sqlx(type_name = "order_status")]
pub enum OrderStatus {
    PENDING,
    PARTIAL,
    FILLED,
    CANCELLED,
    EXPIRED,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Eq, PartialEq)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub outcome_id: Uuid,
    #[sqlx(rename = "type")]
    pub order_type: OrderType,
    pub shares: Decimal,
    pub remaining_shares: Decimal,
    pub price: Decimal,
    pub status: OrderStatus,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, sqlx::Type)]
pub struct Trade {
    pub id: Uuid,
    pub market_id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub shares: Decimal,
    pub price: Decimal,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, sqlx::Type)]
pub struct ResolvedMarket {
    pub id: Uuid,
    pub market_id: Uuid,
    pub winning_outcome_id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub picture: Option<String>,
    pub mobile_no: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance: Decimal,
    pub locked_balance: Decimal,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type")]
pub enum TransactionType {
    DEPOSIT,
    WITHDRAW,
    BUY,
    SELL,
    REFUND,
    PAYOUT,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    #[sqlx(rename = "type")]
    pub transaction_type: TransactionType,
    pub amount: Decimal,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Holding {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub outcome_id: Uuid,
    pub shares: Decimal,
    pub locked_shares: Decimal,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOutcomes {
    pub first_outcome: Outcome,
    pub second_outcome: Outcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MatcherMessage {
    PlaceOrder {
        order: Order,
    },
    CancelOrder {
        order: Order,
    },
    CreateMarket {
        market: Market,
        outcomes: MarketOutcomes,
    },
    RemoveMarket {
        market: Market,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderFeed {
    pub market_id: Uuid,
    pub outcome_id: Uuid,
    pub quantity: Decimal,
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    JoinMarket { market_id: Uuid },
    LeaveMarket { market_id: Uuid },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    JoinedMarket { market_id: Uuid },
    LeftMarket { market_id: Uuid },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FeedMessage {
    OrderFeed { feed: OrderFeed },
    CreateMarket { market_id: Uuid },
    RemoveMarket { market_id: Uuid },
}
