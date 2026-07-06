use crate::model::{HoldingRow, OrderRow, TradeRow, TransactionRow};
use clickhouse::Client;

const CREATE_ORDERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS orders
(
    id UUID,
    user_id UUID,
    market_id UUID,
    outcome_id UUID,

    side Enum8(
        'BUY' = 1,
        'SELL' = 2
    ),

    shares Decimal64(8),
    remaining_shares Decimal64(8),
    price Decimal64(8),

    status Enum8(
        'PENDING' = 1,
        'PARTIAL' = 2,
        'FILLED' = 3,
        'CANCELLED' = 4,
        'EXPIRED' = 5
    ),

    created_at DateTime64(3),
    updated_at DateTime64(3)
)
ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY toYYYYMM(created_at)
ORDER BY (market_id, outcome_id, created_at, id);
"#;

const CREATE_TRADES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS trades
(
    id UUID,
    market_id UUID,
    buy_order_id UUID,
    sell_order_id UUID,

    shares Decimal64(8),
    price Decimal64(8),

    created_at DateTime64(3)
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(created_at)
ORDER BY (market_id, created_at, id);
"#;

const CREATE_TRANSACTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS transactions
(
    id UUID,
    wallet_id UUID,

    type Enum8(
        'DEPOSIT' = 1,
        'WITHDRAW' = 2,
        'BUY' = 3,
        'SELL' = 4,
        'REFUND' = 5,
        'PAYOUT' = 6
    ),

    amount Decimal64(8),

    created_at DateTime64(3)
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(created_at)
ORDER BY (wallet_id, created_at, id);
"#;

const CREATE_HOLDINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS holdings
(
    id UUID,
    user_id UUID,
    market_id UUID,
    outcome_id UUID,

    shares Decimal64(8),
    locked_shares Decimal64(8),

    created_at DateTime64(3),
    updated_at DateTime64(3)
)
ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY toYYYYMM(updated_at)
ORDER BY (user_id, market_id, outcome_id);
"#;

#[derive(Clone)]
pub struct CHClient {
    pub client: Client,
}

impl CHClient {
    pub fn new(url: &str, user: &str, password: &str, database: &str) -> Self {
        Self {
            client: Client::default()
                .with_url(url)
                .with_user(user)
                .with_password(password)
                .with_database(database),
        }
    }

    pub async fn create_tables(&self) -> Result<(), clickhouse::error::Error> {
        self.client.query(CREATE_ORDERS_TABLE).execute().await?;
        self.client.query(CREATE_TRADES_TABLE).execute().await?;
        self.client
            .query(CREATE_TRANSACTIONS_TABLE)
            .execute()
            .await?;
        self.client.query(CREATE_HOLDINGS_TABLE).execute().await?;

        Ok(())
    }

    pub async fn insert_order(&self, order: &OrderRow) -> Result<(), clickhouse::error::Error> {
        let mut insert = self.client.insert("orders")?;

        insert.write(order).await?;
        insert.end().await?;

        Ok(())
    }

    pub async fn insert_trade(&self, trade: &TradeRow) -> Result<(), clickhouse::error::Error> {
        let mut insert = self.client.insert("trades")?;

        insert.write(trade).await?;
        insert.end().await?;

        Ok(())
    }

    pub async fn insert_transaction(
        &self,
        transaction: &TransactionRow,
    ) -> Result<(), clickhouse::error::Error> {
        let mut insert = self.client.insert("transactions")?;

        insert.write(transaction).await?;
        insert.end().await?;

        Ok(())
    }

    pub async fn insert_holding(
        &self,
        holding: &HoldingRow,
    ) -> Result<(), clickhouse::error::Error> {
        let mut insert = self.client.insert("holdings")?;

        insert.write(holding).await?;
        insert.end().await?;

        Ok(())
    }
}
