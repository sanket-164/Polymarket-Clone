use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    database::client::PGClient,
    model::{Admin, Market, MarketStatus, MarketWithOutcomes, Order, OrderSide, Outcome, Wallet},
};
use sqlx::{Row, postgres::PgRow};
use uuid::Uuid;

use crate::dto::CreateMarketDTO;

#[async_trait]
pub trait AccountExt {
    async fn get_admin_by_id(&self, admin_id: Uuid) -> Result<Option<Admin>, sqlx::Error>;
}

#[async_trait]
pub trait MarketExt {
    async fn create_market(
        &self,
        market: CreateMarketDTO,
        admin_id: Uuid,
    ) -> Result<MarketWithOutcomes, sqlx::Error>;
    async fn insert_sell_order(
        &self,
        outcome: Outcome,
        admin_id: Uuid,
    ) -> Result<Order, sqlx::Error>;
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
    async fn get_market_by_id(&self, market_id: Uuid) -> Result<Option<Market>, sqlx::Error>;
    async fn get_market_details(
        &self,
        market_id: Uuid,
    ) -> Result<Option<MarketWithOutcomes>, sqlx::Error>;
    async fn get_market_outcome(
        &self,
        market_id: Uuid,
        outcome_id: Uuid,
    ) -> Result<Option<Outcome>, sqlx::Error>;
    async fn resolve_market(
        &self,
        admin_id: Uuid,
        market_id: Uuid,
        winning_outcome_id: Uuid,
    ) -> Result<(), sqlx::Error>;
}

#[async_trait]
pub trait WalletExt {
    async fn get_wallet(&self, user_id: Uuid) -> Result<Wallet, sqlx::Error>;
}

#[async_trait]
impl AccountExt for PGClient {
    async fn get_admin_by_id(&self, admin_id: Uuid) -> Result<Option<Admin>, sqlx::Error> {
        let query =
            r#"SELECT id, name, email, password, created_at, updated_at FROM admins WHERE id = $1"#;

        let admin = sqlx::query_as::<_, Admin>(query)
            .bind(admin_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(admin)
    }
}

#[async_trait]
impl MarketExt for PGClient {
    async fn create_market(
        &self,
        market_data: CreateMarketDTO,
        admin_id: Uuid,
    ) -> Result<MarketWithOutcomes, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let insert_market_query = "
            INSERT INTO market (title, description, category, start_at, close_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, title, description, category, start_at, close_at, status, created_at, updated_at, deleted_at
        ";

        let market: Market = sqlx::query_as(insert_market_query)
            .bind(&market_data.title)
            .bind(&market_data.desciption)
            .bind(&market_data.category)
            .bind(market_data.start_at)
            .bind(market_data.close_at)
            .fetch_one(&mut *tx)
            .await?;

        let insert_outcome_query = "
            INSERT INTO outcome (market_id, label, start_price, current_price, total_shares)
            VALUES ($1, $2, $3, $3, $4)
            RETURNING id, market_id, label, start_price, current_price, total_shares, created_at, updated_at
        ";

        let first_outcome: Outcome = sqlx::query_as(insert_outcome_query)
            .bind(market.id)
            .bind(&market_data.first_outcome.label)
            .bind(market_data.first_outcome.start_price)
            .bind(market_data.first_outcome.total_shares)
            .fetch_one(&mut *tx)
            .await?;

        let second_outcome: Outcome = sqlx::query_as(insert_outcome_query)
            .bind(market.id)
            .bind(&market_data.second_outcome.label)
            .bind(market_data.second_outcome.start_price)
            .bind(market_data.second_outcome.total_shares)
            .fetch_one(&mut *tx)
            .await?;

        let insert_holdings_query = "
            INSERT INTO holdings (user_id, market_id, outcome_id, shares, locked_shares)
            VALUES
            ($1, $2, $3, 0.00, $4),
            ($1, $5, $6, 0.00, $7)
        ";

        sqlx::query(insert_holdings_query)
            .bind(admin_id)
            .bind(market.id)
            .bind(first_outcome.id)
            .bind(first_outcome.total_shares)
            .bind(market.id)
            .bind(second_outcome.id)
            .bind(second_outcome.total_shares)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(MarketWithOutcomes {
            market,
            first_outcome,
            second_outcome,
        })
    }

    async fn insert_sell_order(
        &self,
        outcome: Outcome,
        admin_id: Uuid,
    ) -> Result<Order, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let insert_order_query = "INSERT INTO orders (user_id, market_id, outcome_id, side, shares, remaining_shares, price)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, user_id, market_id, outcome_id, side, shares, remaining_shares, price, status, created_at, updated_at";

        let order: Order = sqlx::query_as(insert_order_query)
            .bind(admin_id)
            .bind(outcome.market_id)
            .bind(outcome.id)
            .bind(OrderSide::SELL)
            .bind(outcome.total_shares)
            .bind(outcome.total_shares)
            .bind(outcome.start_price)
            .fetch_one(&mut *tx)
            .await?;

        let cost = outcome.total_shares * outcome.start_price;

        sqlx::query(
            "UPDATE wallets SET balance = balance - $1, updated_at = NOW() WHERE user_id = $2",
        )
        .bind(cost)
        .bind(admin_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(order)
    }

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
            query.push_str(&format!(" AND category = ${param_index}"));
            param_index += 1;
        }
        if start_after.is_some() {
            query.push_str(&format!(" AND start_at >= ${param_index}"));
            param_index += 1;
        }
        if start_before.is_some() {
            query.push_str(&format!(" AND start_at <= ${param_index}"));
            param_index += 1;
        }
        if close_after.is_some() {
            query.push_str(&format!(" AND close_at >= ${param_index}"));
            param_index += 1;
        }
        if close_before.is_some() {
            query.push_str(&format!(" AND close_at <= ${param_index}"));
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

    async fn get_market_by_id(&self, market_id: Uuid) -> Result<Option<Market>, sqlx::Error> {
        let query = r#"
            SELECT id, title, description, category, start_at, close_at, status, created_at, updated_at, deleted_at
            FROM market
            WHERE id = $1 AND deleted_at IS NULL"#;

        let market = sqlx::query_as::<_, Market>(query)
            .bind(market_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(market)
    }

    async fn get_market_details(
        &self,
        market_id: Uuid,
    ) -> Result<Option<MarketWithOutcomes>, sqlx::Error> {
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
            WHERE m.id = $1 AND m.deleted_at IS NULL
            ORDER BY o.created_at ASC
        ";

        let rows = sqlx::query(query)
            .bind(market_id)
            .fetch_all(&self.pool)
            .await?;

        if rows.len() < 2 {
            return Ok(None);
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

        Ok(Some(MarketWithOutcomes {
            market,
            first_outcome: parse_outcome(&rows[0]),
            second_outcome: parse_outcome(&rows[1]),
        }))
    }

    async fn get_market_outcome(
        &self,
        market_id: Uuid,
        outcome_id: Uuid,
    ) -> Result<Option<Outcome>, sqlx::Error> {
        let query = r#"
            SELECT id, market_id, label, start_price, current_price, total_shares, created_at, updated_at
            FROM outcome
            WHERE id = $1 AND market_id = $2
        "#;

        let outcome = sqlx::query_as::<_, Outcome>(query)
            .bind(outcome_id)
            .bind(market_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(outcome)
    }

    async fn resolve_market(
        &self,
        admin_id: Uuid,
        market_id: Uuid,
        winning_outcome_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // 1. Refund balance of unmatched buy orders
        // Reduce loacked_balance and return balance (remaining_shares * price)
        sqlx::query(
            "UPDATE wallets w
            SET balance        = w.balance + unmatched_buys.total,
                locked_balance = w.locked_balance - unmatched_buys.total,
                updated_at = $1
            FROM (
                SELECT user_id, SUM(remaining_shares * price) AS total
                FROM orders
                WHERE market_id = $2
                AND side      = 'BUY'
                AND status    IN ('PENDING', 'PARTIAL')
                GROUP BY user_id
            ) AS unmatched_buys
            WHERE w.user_id = unmatched_buys.user_id",
        )
        .bind(Utc::now())
        .bind(market_id)
        .execute(&mut *tx)
        .await?;

        // 2. Return unmatched sell order shares back to holdings
        sqlx::query(
            "UPDATE holdings h
            SET shares        = h.shares + unmatched_sells.total_shares,
                locked_shares = h.locked_shares - unmatched_sells.total_shares,
                updated_at = $1
            FROM (
                SELECT user_id, outcome_id, SUM(remaining_shares) AS total_shares
                FROM orders
                WHERE market_id = $2
                AND side      = 'SELL'
                AND status    IN ('PENDING', 'PARTIAL')
                GROUP BY user_id, outcome_id
            ) AS unmatched_sells
            WHERE h.user_id    = unmatched_sells.user_id
            AND h.market_id  = $2
            AND h.outcome_id = unmatched_sells.outcome_id",
        )
        .bind(Utc::now())
        .bind(market_id)
        .execute(&mut *tx)
        .await?;

        // 3. Admin collects losing shares (each share = 1 value)
        sqlx::query(
            "UPDATE wallets w
            SET balance = w.balance + losers.total
            FROM (
                SELECT SUM(shares) AS total
                FROM holdings
                WHERE market_id = $1
                AND outcome_id != $2
            ) AS losers
            WHERE w.user_id = $3",
        )
        .bind(market_id)
        .bind(winning_outcome_id)
        .bind(admin_id)
        .execute(&mut *tx)
        .await?;

        // 4. Admin pays out winning shares (each share = 1 value)
        sqlx::query(
            "UPDATE wallets w
            SET balance = w.balance - winners.total,
            updated_at = $1
            FROM (
                SELECT SUM(shares) AS total
                FROM holdings
                WHERE market_id  = $2
                AND outcome_id = $3
            ) AS winners
            WHERE w.user_id = $4",
        )
        .bind(Utc::now())
        .bind(market_id)
        .bind(winning_outcome_id)
        .bind(admin_id)
        .execute(&mut *tx)
        .await?;

        // 5. Credit each winner 1 per share held
        sqlx::query(
            "UPDATE wallets w
            SET balance = w.balance + winners.total,
            updated_at = $1
            FROM (
                SELECT user_id, shares AS total
                FROM holdings
                WHERE market_id  = $2
                AND outcome_id = $3
            ) AS winners
            WHERE w.user_id = winners.user_id",
        )
        .bind(Utc::now())
        .bind(market_id)
        .bind(winning_outcome_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO transactions (wallet_id, amount, type)
            SELECT w.id, h.shares, 'PAYOUT'
            FROM holdings h
            JOIN wallets w ON w.user_id = h.user_id
            WHERE h.market_id = $1
            AND h.outcome_id = $2
            AND h.shares > 0.0",
        )
        .bind(market_id)
        .bind(winning_outcome_id)
        .execute(&mut *tx)
        .await?;

        // 6. Expire all open orders for this market
        sqlx::query(
            "UPDATE orders
            SET status = 'EXPIRED'
            WHERE market_id = $1
            AND status IN ('PENDING', 'PARTIAL')",
        )
        .bind(market_id)
        .execute(&mut *tx)
        .await?;

        // 7. Mark market resolved
        sqlx::query(
            "UPDATE market
            SET status = 'RESOLVED',
            updated_at = $1
            WHERE id = $2",
        )
        .bind(Utc::now())
        .bind(market_id)
        .execute(&mut *tx)
        .await?;

        // 8. Record resolution
        sqlx::query(
            "INSERT INTO resolved_markets (market_id, winning_outcome_id)
            VALUES ($1, $2)",
        )
        .bind(market_id)
        .bind(winning_outcome_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait]
impl WalletExt for PGClient {
    async fn get_wallet(&self, user_id: Uuid) -> Result<Wallet, sqlx::Error> {
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
}
