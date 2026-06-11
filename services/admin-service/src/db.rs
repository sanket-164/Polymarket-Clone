use async_trait::async_trait;
use chrono::Utc;
use common::{
    database::client::PGClient,
    model::{Admin, Market, MarketWithOutcomes, Order, OrderType, Outcome},
    validation::admin_dto::CreateMarketDTO,
};
use uuid::Uuid;

#[async_trait]
pub trait AccountExt {
    async fn get_admin_by_id(&self, admin_id: Uuid) -> Result<Option<Admin>, sqlx::Error>;
    async fn update_admin<T: Into<String> + Send>(
        &self,
        admin_id: Uuid,
        name: T,
        email: T,
    ) -> Result<Admin, sqlx::Error>;
}

#[async_trait]
pub trait MarketExt {
    async fn get_market_by_id(
        &self,
        market_id: Uuid,
    ) -> Result<Option<MarketWithOutcomes>, sqlx::Error>;
    async fn create_market(
        &self,
        market: CreateMarketDTO,
    ) -> Result<MarketWithOutcomes, sqlx::Error>;
    async fn insert_sell_order(&self, outcome: Outcome) -> Result<Order, sqlx::Error>;
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

    async fn update_admin<T: Into<String> + Send>(
        &self,
        admin_id: Uuid,
        name: T,
        email: T,
    ) -> Result<Admin, sqlx::Error> {
        let name: String = name.into();
        let email = email.into();

        let admin = sqlx::query_as!(
            Admin,
            r#"
            UPDATE admins 
            SET name = $1, email = $2, updated_at = $3
            WHERE id = $4
            RETURNING id, name, email, password, created_at, updated_at
            "#,
            name,
            email,
            Utc::now(),
            admin_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(admin)
    }
}

#[async_trait]
impl MarketExt for PGClient {
    async fn get_market_by_id(
        &self,
        market_id: Uuid,
    ) -> Result<Option<MarketWithOutcomes>, sqlx::Error> {
        let market: Option<Market> = sqlx::query_as(
            "SELECT id, title, description, category, start_at, close_at, status, created_at, updated_at, deleted_at
            FROM market
            WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(market_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(market) = market else {
            return Ok(None);
        };

        let outcomes: Vec<Outcome> = sqlx::query_as(
            "SELECT id, market_id, label, start_price, current_price, total_shares, created_at, updated_at
            FROM 
            WHERE market_id = $1"
        )
        .bind(market_id)
        .fetch_all(&self.pool)
        .await?;

        let mut outcomes_iter = outcomes.into_iter();

        let first_outcome = outcomes_iter.next().ok_or(sqlx::Error::RowNotFound)?;
        let second_outcome = outcomes_iter.next().ok_or(sqlx::Error::RowNotFound)?;

        Ok(Some(MarketWithOutcomes {
            market,
            first_outcome,
            second_outcome,
        }))
    }

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
        let insert_order_query = "INSERT INTO orders (user_id, market_id, outcome_id, type, shares, remaining_shares, price)
               VALUES ('11111111-1111-1111-1111-111111111111', $1, $2, $3, $4, $5, $6)
               RETURNING id, user_id, market_id, outcome_id, type, shares, remaining_shares, price, status, created_at, updated_at";

        let order = sqlx::query_as(insert_order_query)
            .bind(outcome.market_id)
            .bind(outcome.id)
            .bind(OrderType::SELL)
            .bind(outcome.total_shares)
            .bind(outcome.total_shares)
            .bind(outcome.start_price)
            .fetch_one(&self.pool)
            .await?;

        Ok(order)
    }
}
