use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    database::client::PGClient,
    model::{Holding, Market, Order, OrderSide, OrderStatus, Outcome, User, Wallet},
};
use rust_decimal::Decimal;
use uuid::Uuid;

#[async_trait]
pub trait OrderExt {
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn get_market_by_id(&self, market_id: Uuid) -> Result<Option<Market>, sqlx::Error>;
    async fn get_market_outcome(
        &self,
        outcome_id: Uuid,
        market_id: Uuid,
    ) -> Result<Option<Outcome>, sqlx::Error>;
    async fn get_market_outcomes(&self, market_id: Uuid) -> Result<Vec<Outcome>, sqlx::Error>;
    async fn get_user_wallet(&self, user_id: Uuid) -> Result<Wallet, sqlx::Error>;
    async fn get_user_holding(
        &self,
        user_id: Uuid,
        outcome_id: Uuid,
    ) -> Result<Option<Holding>, sqlx::Error>;
    async fn get_user_orders(
        &self,
        user_id: Uuid,
        market_id: Option<Uuid>,
        side: Option<OrderSide>,
        status: Option<OrderStatus>,
        before: Option<DateTime<Utc>>,
        after: Option<DateTime<Utc>>,
        order_by: String,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<Order>, sqlx::Error>;
    async fn buy_order(
        &self,
        user_id: Uuid,
        market_id: Uuid,
        outcome_id: Uuid,
        shares: Decimal,
        price: Decimal,
    ) -> Result<Order, sqlx::Error>;
    async fn sell_order(
        &self,
        user_id: Uuid,
        market_id: Uuid,
        outcome_id: Uuid,
        shares: Decimal,
        price: Decimal,
    ) -> Result<Order, sqlx::Error>;
}

#[async_trait]
impl OrderExt for PGClient {
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, name, email, password, picture, mobile_no, created_at, updated_at FROM users WHERE id = $1"#,
            user_id
        ).fetch_optional(&self.pool).await?;

        Ok(user)
    }

    async fn get_market_by_id(&self, market_id: Uuid) -> Result<Option<Market>, sqlx::Error> {
        let query = r#"
            SELECT id, title, description, category, start_at, close_at, status, created_at, updated_at, deleted_at
            FROM market
            WHERE id = $1"#;

        let market = sqlx::query_as::<_, Market>(query)
            .bind(market_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(market)
    }

    async fn get_market_outcome(
        &self,
        outcome_id: Uuid,
        market_id: Uuid,
    ) -> Result<Option<Outcome>, sqlx::Error> {
        let query = r#"
            SELECT id, market_id, label, start_price, current_price, total_shares, created_at, updated_at
            FROM outcome
            WHERE id = $1 AND market_id = $2"#;

        let outcome = sqlx::query_as::<_, Outcome>(query)
            .bind(outcome_id)
            .bind(market_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(outcome)
    }

    async fn get_market_outcomes(&self, market_id: Uuid) -> Result<Vec<Outcome>, sqlx::Error> {
        let outcomes = sqlx::query_as::<_, Outcome>(r#"SELECT id, market_id, label, start_price, current_price, total_shares, created_at, updated_at FROM outcome WHERE market_id = $1"#)
            .bind(market_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(outcomes)
    }

    async fn get_user_wallet(&self, user_id: Uuid) -> Result<Wallet, sqlx::Error> {
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

    async fn get_user_holding(
        &self,
        user_id: Uuid,
        outcome_id: Uuid,
    ) -> Result<Option<Holding>, sqlx::Error> {
        let query = r#"
            SELECT id, user_id, market_id, outcome_id, shares, locked_shares, created_at, updated_at
            FROM holdings
            WHERE user_id = $1 AND outcome_id = $2"#;

        let holding = sqlx::query_as::<_, Holding>(query)
            .bind(user_id)
            .bind(outcome_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(holding)
    }

    async fn get_user_orders(
        &self,
        user_id: Uuid,
        market_id: Option<Uuid>,
        side: Option<OrderSide>,
        status: Option<OrderStatus>,
        before: Option<DateTime<Utc>>,
        after: Option<DateTime<Utc>>,
        order_by: String,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<Order>, sqlx::Error> {
        let mut query = String::from(
            "SELECT id, user_id, market_id, outcome_id, side, shares, remaining_shares, price, status, created_at, updated_at
            FROM orders
            WHERE user_id = $1"
        );

        let mut param_index = 2;

        if market_id.is_some() {
            query.push_str(&format!(" AND market_id = ${}", param_index));
            param_index += 1;
        }
        if side.is_some() {
            query.push_str(&format!(" AND type = ${}", param_index));
            param_index += 1;
        }
        if status.is_some() {
            query.push_str(&format!(" AND status = ${}", param_index));
            param_index += 1;
        }
        if before.is_some() {
            query.push_str(&format!(" AND created_at < ${}", param_index));
            param_index += 1;
        }
        if after.is_some() {
            query.push_str(&format!(" AND created_at > ${}", param_index));
            param_index += 1;
        }

        query.push_str(&format!(
            " ORDER BY {} LIMIT ${} OFFSET ${}",
            order_by,
            param_index,
            param_index + 1
        ));

        let mut q = sqlx::query_as::<_, Order>(&query).bind(user_id);

        if let Some(mid) = market_id {
            q = q.bind(mid);
        }
        if let Some(ot) = side {
            q = q.bind(ot);
        }
        if let Some(s) = status {
            q = q.bind(s);
        }
        if let Some(b) = before {
            q = q.bind(b);
        }
        if let Some(a) = after {
            q = q.bind(a);
        }

        q = q.bind(limit).bind(skip);

        let orders = q.fetch_all(&self.pool).await?;

        Ok(orders)
    }

    async fn buy_order(
        &self,
        user_id: Uuid,
        market_id: Uuid,
        outcome_id: Uuid,
        shares: Decimal,
        price: Decimal,
    ) -> Result<Order, sqlx::Error> {
        let cost = price * shares;
        let mut tx = self.pool.begin().await?;

        let order = sqlx::query_as::<_, Order>(
            r#"WITH deduct_wallet AS (
               UPDATE wallets
               SET    balance         = balance         - $1,
                      locked_balance  = locked_balance  + $1,
                      updated_at      = NOW()
               WHERE  user_id = $2
           ),
           upsert_holding AS (
               INSERT INTO holdings (user_id, market_id, outcome_id, shares, locked_shares)
               VALUES ($2, $3, $4, 0, 0)
               ON CONFLICT (user_id, market_id, outcome_id) DO NOTHING
           )
           INSERT INTO orders
               (user_id, market_id, outcome_id, side, shares, remaining_shares, price)
           VALUES ($2, $3, $4, $5, $6, $6, $7)
           RETURNING
               id, user_id, market_id, outcome_id, side,
               shares, remaining_shares, price, status,
               created_at, updated_at"#,
        )
        .bind(cost)
        .bind(user_id)
        .bind(market_id)
        .bind(outcome_id)
        .bind(OrderSide::BUY)
        .bind(shares) // (shares & remaining_shares)
        .bind(price)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(order)
    }

    async fn sell_order(
        &self,
        user_id: Uuid,
        market_id: Uuid,
        outcome_id: Uuid,
        shares: Decimal,
        price: Decimal,
    ) -> Result<Order, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let order = sqlx::query_as::<_, Order>(
            r#"WITH lock_shares AS (
               UPDATE holdings
               SET    shares        = shares        - $1,
                      locked_shares = locked_shares + $1,
                      updated_at    = NOW()
               WHERE  user_id = $2 AND market_id = $3 AND outcome_id = $4
           )
           INSERT INTO orders
               (user_id, market_id, outcome_id, side, shares, remaining_shares, price)
           VALUES ($2, $3, $4, $5, $1, $1, $6)
           RETURNING
               id, user_id, market_id, outcome_id, side,
               shares, remaining_shares, price, status,
               created_at, updated_at"#,
        )
        .bind(shares) // (shares & remaining_shares)
        .bind(user_id)
        .bind(market_id)
        .bind(outcome_id)
        .bind(OrderSide::SELL)
        .bind(price)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(order)
    }
}
