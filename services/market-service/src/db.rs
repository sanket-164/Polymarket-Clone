use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    database::client::PGClient,
    model::{Admin, Market, MarketStatus, MarketWithOutcomes, Order, OrderSide, Outcome},
    validation::admin_dto::CreateMarketDTO,
};
use sqlx::{Row, postgres::PgRow};
use uuid::Uuid;

#[async_trait]
pub trait AccountExt {
    async fn get_admin_by_id(&self, admin_id: Uuid) -> Result<Option<Admin>, sqlx::Error>;
}

#[async_trait]
pub trait MarketExt {
    async fn create_market(
        &self,
        market: CreateMarketDTO,
    ) -> Result<MarketWithOutcomes, sqlx::Error>;
    async fn insert_sell_order(&self, outcome: Outcome) -> Result<Order, sqlx::Error>;
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
    async fn get_market_details(
        &self,
        market_id: Uuid,
    ) -> Result<Option<MarketWithOutcomes>, sqlx::Error>;
}

#[async_trait]
impl AccountExt for PGClient {
    async fn get_admin_by_id(&self, admin_id: Uuid) -> Result<Option<Admin>, sqlx::Error> {
        let admin = sqlx::query_as!(
            Admin,
            r#"SELECT id, name, email, password, created_at, updated_at FROM admins WHERE id = $1"#,
            admin_id
        )
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
            ('11111111-1111-1111-1111-111111111111', $1, $2, 0.00, $3),
            ('11111111-1111-1111-1111-111111111111', $4, $5, 0.00, $6)
        ";

        sqlx::query(insert_holdings_query)
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

    async fn insert_sell_order(&self, outcome: Outcome) -> Result<Order, sqlx::Error> {
        let insert_order_query = "INSERT INTO orders (user_id, market_id, outcome_id, side, shares, remaining_shares, price)
               VALUES ('11111111-1111-1111-1111-111111111111', $1, $2, $3, $4, $5, $6)
               RETURNING id, user_id, market_id, outcome_id, side, shares, remaining_shares, price, status, created_at, updated_at";

        let order = sqlx::query_as(insert_order_query)
            .bind(outcome.market_id)
            .bind(outcome.id)
            .bind(OrderSide::SELL)
            .bind(outcome.total_shares)
            .bind(outcome.total_shares)
            .bind(outcome.start_price)
            .fetch_one(&self.pool)
            .await?;

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
        WHERE m.id = $1
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
}
