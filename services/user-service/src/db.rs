use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    database::client::PGClient,
    model::{
        Holding, Market, MarketStatus, MarketWithOutcomes, Outcome, Transaction, TransactionType,
        User, Wallet,
    },
    validation::user_dto::HoldingDetails,
};
use rust_decimal::Decimal;
use sqlx::{Row, postgres::PgRow};
use uuid::Uuid;

#[async_trait]
pub trait AccountExt {
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn create_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error>;
    async fn update_user<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        name: T,
        email: T,
        mobile_no: Option<T>,
    ) -> Result<User, sqlx::Error>;
    async fn update_user_picture<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        picture: T,
    ) -> Result<User, sqlx::Error>;
}

#[async_trait]
pub trait WalletExt {
    async fn get_balance(&self, user_id: Uuid) -> Result<Wallet, sqlx::Error>;
    async fn deposite_balance(&self, user_id: Uuid, amount: Decimal)
    -> Result<Wallet, sqlx::Error>;
    async fn withdraw_balance(&self, user_id: Uuid, amount: Decimal)
    -> Result<Wallet, sqlx::Error>;
    async fn get_transactions(
        &self,
        wallet_id: Uuid,
        transaction_type: Option<TransactionType>,
        order_by: String,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<Transaction>, sqlx::Error>;
}

#[async_trait]
pub trait MarketExt {
    async fn get_markets(
        &self,
        status: MarketStatus,
        category: Option<String>,
        start_after: Option<DateTime<Utc>>,
        start_before: Option<DateTime<Utc>>,
        close_after: Option<DateTime<Utc>>,
        close_before: Option<DateTime<Utc>>,
        order_by: String,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<Market>, sqlx::Error>;
    async fn get_market_details(&self, market_id: Uuid) -> Result<MarketWithOutcomes, sqlx::Error>;
}

#[async_trait]
pub trait HoldingExt {
    async fn get_user_holdings(
        &self,
        user_id: Uuid,
        order_by: String,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<HoldingDetails>, sqlx::Error>;
}

#[async_trait]
impl AccountExt for PGClient {
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, name, email, password, picture, mobile_no, created_at, updated_at FROM users WHERE id = $1"#,
            user_id
        ).fetch_optional(&self.pool).await?;

        Ok(user)
    }

    async fn create_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO users (name, email, password)
            VALUES ($1, $2, $3)
            RETURNING id, name, email, password, picture, mobile_no, created_at, updated_at"#,
            name.into(),
            email.into(),
            password.into()
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"INSERT INTO wallets (user_id)
            VALUES ($1)"#,
            user.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(user)
    }

    async fn update_user<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        name: T,
        email: T,
        mobile_no: Option<T>,
    ) -> Result<User, sqlx::Error> {
        let name = name.into();
        let email = email.into();
        let mobile_no = mobile_no.map(|m| m.into());

        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users 
            SET name = $1, email = $2, mobile_no = COALESCE($3, mobile_no), updated_at = $4
            WHERE id = $5
            RETURNING id, name, email, password, picture, mobile_no, created_at, updated_at
            "#,
            name,
            email,
            mobile_no,
            Utc::now(),
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user_picture<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        picture: T,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users SET picture = $1, updated_at = $2
            WHERE id = $3 
            RETURNING id, name, email, password, picture, mobile_no, created_at, updated_at
            "#,
            picture.into(),
            Utc::now(),
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}

#[async_trait]
impl WalletExt for PGClient {
    async fn get_balance(&self, user_id: Uuid) -> Result<Wallet, sqlx::Error> {
        let wallet = sqlx::query_as!(
            Wallet,
            r#"
            SELECT id, user_id, balance as "balance!: Decimal", locked_balance as "locked_balance!: Decimal", created_at, updated_at
            FROM wallets 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(wallet)
    }

    async fn deposite_balance(
        &self,
        user_id: Uuid,
        amount: Decimal,
    ) -> Result<Wallet, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let wallet = sqlx::query_as!(
            Wallet,
            r#"
            UPDATE wallets SET balance = balance + $1, updated_at = $2
            WHERE user_id = $3
            RETURNING id, user_id, balance as "balance!: Decimal", locked_balance as "locked_balance!: Decimal", created_at, updated_at
            "#,
            amount,
            Utc::now(),
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO transactions (wallet_id, amount, type) 
            VALUES ($1, $2, 'DEPOSIT')
            "#,
            wallet.id,
            amount
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(wallet)
    }

    async fn withdraw_balance(
        &self,
        user_id: Uuid,
        amount: Decimal,
    ) -> Result<Wallet, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let wallet = sqlx::query_as!(
            Wallet,
            r#"
            UPDATE wallets SET balance = balance - $1, updated_at = $2
            WHERE user_id = $3
            RETURNING id, user_id, balance as "balance!: Decimal", locked_balance as "locked_balance!: Decimal", created_at, updated_at
            "#,
            amount,
            Utc::now(),
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO transactions (wallet_id, amount, type) 
            VALUES ($1, $2, 'WITHDRAW')
            "#,
            wallet.id,
            amount
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(wallet)
    }

    async fn get_transactions(
        &self,
        user_id: Uuid,
        transaction_type: Option<TransactionType>,
        order_by: String,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<Transaction>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT 
                id, 
                wallet_id, 
                amount, 
                type,
                created_at 
            FROM transactions 
            WHERE wallet_id = (SELECT id FROM wallets WHERE user_id = $1)
            AND ($2::transaction_type IS NULL OR type = $2)
            ORDER BY {order_by}
            LIMIT $3 OFFSET $4
            "#
        );

        let transactions = sqlx::query_as::<_, Transaction>(&query)
            .bind(user_id)
            .bind(transaction_type)
            .bind(limit)
            .bind(skip)
            .fetch_all(&self.pool)
            .await?;

        Ok(transactions)
    }
}

#[async_trait]
impl MarketExt for PGClient {
    async fn get_markets(
        &self,
        status: MarketStatus,
        category: Option<String>,
        start_after: Option<DateTime<Utc>>,
        start_before: Option<DateTime<Utc>>,
        close_after: Option<DateTime<Utc>>,
        close_before: Option<DateTime<Utc>>,
        order_by: String,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<Market>, sqlx::Error> {
        let mut query = String::from(
        "SELECT id, title, description, category, start_at, close_at, status, created_at, updated_at, deleted_at
        FROM market
        WHERE deleted_at IS NULL AND status = $1",
    );

        let mut param_index = 2;

        if category.is_some() {
            query.push_str(&format!(" AND category = ${}", param_index));
            param_index += 1;
        }
        if start_after.is_some() {
            query.push_str(&format!(" AND start_at >= ${}", param_index));
            param_index += 1;
        }
        if start_before.is_some() {
            query.push_str(&format!(" AND start_at <= ${}", param_index));
            param_index += 1;
        }
        if close_after.is_some() {
            query.push_str(&format!(" AND close_at >= ${}", param_index));
            param_index += 1;
        }
        if close_before.is_some() {
            query.push_str(&format!(" AND close_at <= ${}", param_index));
            param_index += 1;
        }

        query.push_str(&format!(
            " ORDER BY {} LIMIT ${} OFFSET ${}",
            order_by,
            param_index,
            param_index + 1
        ));

        let mut q = sqlx::query_as::<_, Market>(&query).bind(status);

        if let Some(c) = category {
            q = q.bind(c);
        }
        if let Some(sa) = start_after {
            q = q.bind(sa);
        }
        if let Some(sb) = start_before {
            q = q.bind(sb);
        }
        if let Some(ca) = close_after {
            q = q.bind(ca);
        }
        if let Some(cb) = close_before {
            q = q.bind(cb);
        }

        q = q.bind(limit).bind(skip);

        let markets = q.fetch_all(&self.pool).await?;

        Ok(markets)
    }

    async fn get_market_details(&self, market_id: Uuid) -> Result<MarketWithOutcomes, sqlx::Error> {
        let query = "
        SELECT
            m.id, m.title, m.description, m.category,
            m.start_at, m.close_at, m.status,
            m.created_at, m.updated_at, m.deleted_at,
            o.id as outcome_id, o.market_id as outcome_market_id, o.label,
            o.start_price, o.current_price, o.total_shares,
            o.created_at as outcome_created_at, o.updated_at as outcome_updated_at
        FROM market m
        JOIN outcome o ON o.market_id = m.id
        WHERE m.id = $1
        ORDER BY o.created_at ASC
    ";

        let rows = sqlx::query(query)
            .bind(market_id)
            .fetch_all(&self.pool)
            .await?;

        if rows.len() < 2 {
            return Err(sqlx::Error::RowNotFound);
        }

        let market = Market {
            id: rows[0].get("id"),
            title: rows[0].get("title"),
            description: rows[0].get("description"),
            category: rows[0].get("category"),
            start_at: rows[0].get("start_at"),
            close_at: rows[0].get("close_at"),
            status: rows[0].get("status"),
            created_at: rows[0].get("created_at"),
            updated_at: rows[0].get("updated_at"),
            deleted_at: rows[0].get("deleted_at"),
        };

        let parse_outcome = |row: &PgRow| Outcome {
            id: row.get("outcome_id"),
            market_id: row.get("outcome_market_id"),
            label: row.get("label"),
            start_price: row.get("start_price"),
            current_price: row.get("current_price"),
            total_shares: row.get("total_shares"),
            created_at: row.get("outcome_created_at"),
            updated_at: row.get("outcome_updated_at"),
        };

        Ok(MarketWithOutcomes {
            market,
            first_outcome: parse_outcome(&rows[0]),
            second_outcome: parse_outcome(&rows[1]),
        })
    }
}

#[async_trait]
impl HoldingExt for PGClient {
    async fn get_user_holdings(
        &self,
        user_id: Uuid,
        order_by: String,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<HoldingDetails>, sqlx::Error> {
        let query = format!(
            "SELECT
            h.id, h.user_id, h.market_id, h.outcome_id, h.shares, h.locked_shares,
            h.created_at, h.updated_at,
            m.id as market_id, m.title, m.description, m.category,
            m.start_at, m.close_at, m.status, m.created_at as market_created_at,
            m.updated_at as market_updated_at, m.deleted_at,
            o.id as outcome_id, o.market_id as outcome_market_id, o.label,
            o.start_price, o.current_price, o.total_shares,
            o.created_at as outcome_created_at, o.updated_at as outcome_updated_at
        FROM holdings h
        JOIN market m ON h.market_id = m.id
        JOIN outcome o ON h.outcome_id = o.id
        WHERE h.user_id = $1
        ORDER BY {} LIMIT $2 OFFSET $3",
            order_by
        );

        let rows = sqlx::query(&query)
            .bind(user_id)
            .bind(limit)
            .bind(skip)
            .fetch_all(&self.pool)
            .await?;

        let holdings = rows
            .iter()
            .map(|row| {
                let holding = Holding {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    market_id: row.get("market_id"),
                    outcome_id: row.get("outcome_id"),
                    shares: row.get("shares"),
                    locked_shares: row.get("locked_shares"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };

                let market = Market {
                    id: row.get("market_id"),
                    title: row.get("title"),
                    description: row.get("description"),
                    category: row.get("category"),
                    start_at: row.get("start_at"),
                    close_at: row.get("close_at"),
                    status: row.get("status"),
                    created_at: row.get("market_created_at"),
                    updated_at: row.get("market_updated_at"),
                    deleted_at: row.get("deleted_at"),
                };

                let outcome = Outcome {
                    id: row.get("outcome_id"),
                    market_id: row.get("outcome_market_id"),
                    label: row.get("label"),
                    start_price: row.get("start_price"),
                    current_price: row.get("current_price"),
                    total_shares: row.get("total_shares"),
                    created_at: row.get("outcome_created_at"),
                    updated_at: row.get("outcome_updated_at"),
                };

                HoldingDetails {
                    holding,
                    market,
                    outcome,
                }
            })
            .collect();

        Ok(holdings)
    }
}
