use async_trait::async_trait;
use chrono::Utc;
use common::{
    database::client::PGClient,
    model::{Holding, Market, Outcome, Transaction, TransactionType, User, Wallet},
};
use rust_decimal::Decimal;
use sqlx::Row;
use uuid::Uuid;

use crate::dto::HoldingDetails;

#[async_trait]
pub trait AccountExt {
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error>;
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
        let query = r#"SELECT id, name, email, password, picture, mobile_no, created_at, updated_at FROM users WHERE id = $1 AND deleted_at IS NULL"#;

        let user = sqlx::query_as::<_, User>(query)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

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

        let query = r#"
            UPDATE users
            SET name = $1, email = $2, mobile_no = COALESCE($3, mobile_no), updated_at = $4
            WHERE id = $5
            RETURNING id, name, email, password, picture, mobile_no, created_at, updated_at
        "#;

        let user = sqlx::query_as::<_, User>(query)
            .bind(name)
            .bind(email)
            .bind(mobile_no)
            .bind(Utc::now())
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    async fn update_user_picture<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        picture: T,
    ) -> Result<User, sqlx::Error> {
        let query = r#"
            UPDATE users SET picture = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, name, email, password, picture, mobile_no, created_at, updated_at
        "#;

        let user = sqlx::query_as::<_, User>(query)
            .bind(picture.into())
            .bind(Utc::now())
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }
}

#[async_trait]
impl WalletExt for PGClient {
    async fn get_balance(&self, user_id: Uuid) -> Result<Wallet, sqlx::Error> {
        let query = r#"
            SELECT id, user_id, balance, locked_balance, created_at, updated_at
            FROM wallets
            WHERE user_id = $1
        "#;

        let wallet = sqlx::query_as::<_, Wallet>(query)
            .bind(user_id)
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

        let query = r#"
                UPDATE wallets SET balance = balance + $1, updated_at = $2
                WHERE user_id = $3
                RETURNING id, user_id, balance, locked_balance, created_at, updated_at
            "#;

        let wallet = sqlx::query_as::<_, Wallet>(query)
            .bind(amount)
            .bind(Utc::now())
            .bind(user_id)
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

        let query = r#"
                UPDATE wallets SET balance = balance - $1, updated_at = $2
                WHERE user_id = $3
                RETURNING id, user_id, balance, locked_balance, created_at, updated_at
            "#;

        let wallet = sqlx::query_as::<_, Wallet>(query)
            .bind(amount)
            .bind(Utc::now())
            .bind(user_id)
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
                m.updated_at as market_updated_at,
                o.id as outcome_id, o.market_id as outcome_market_id, o.label,
                o.start_price, o.current_price, o.total_shares,
                o.created_at as outcome_created_at, o.updated_at as outcome_updated_at
            FROM holdings h
            JOIN market m ON h.market_id = m.id
            JOIN outcome o ON h.outcome_id = o.id
            WHERE h.user_id = $1
            ORDER BY {order_by} LIMIT $2 OFFSET $3"
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
