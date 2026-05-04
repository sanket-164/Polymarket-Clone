use std::cmp::min;

use async_trait::async_trait;
use chrono::Utc;
use common::{
    database::client::PGClient,
    model::{Order, OrderStatus, Trade, TransactionType},
};

#[async_trait]
pub trait TradeExt {
    async fn trade(&self, buy_order: Order, sell_order: Order) -> Result<Trade, sqlx::Error>;
}

#[async_trait]
impl TradeExt for PGClient {
    async fn trade(&self, buy_order: Order, sell_order: Order) -> Result<Trade, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let trade_shares = min(sell_order.remaining_shares, buy_order.remaining_shares);
        let total_cost = sell_order.price * trade_shares;

        let trade: Trade = sqlx::query_as(
            r#"
            INSERT INTO trades (market_id, outcome_id, buy_order_id, sell_order_id, shares, price)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, market_id, outcome_id, buy_order_id, sell_order_id, shares, price, created_at
            "#,
        )
        .bind(buy_order.market_id)
        .bind(buy_order.outcome_id)
        .bind(buy_order.id)
        .bind(sell_order.id)
        .bind(trade_shares)
        .bind(sell_order.price)
        .fetch_one(&mut *tx)
        .await?;

        for order_id in [buy_order.id, sell_order.id] {
            sqlx::query(
                r#"
                UPDATE orders
                SET
                    remaining_shares = remaining_shares - $1,
                    status = CASE
                        WHEN remaining_shares - $1 = 0 THEN $2
                        ELSE $3
                    END,
                    updated_at = $4
                WHERE id = $5
                "#,
            )
            .bind(trade_shares)
            .bind(OrderStatus::FILLED)
            .bind(OrderStatus::PARTIAL)
            .bind(Utc::now())
            .bind(order_id)
            .execute(&mut *tx)
            .await?;
        }

        // Buyer
        let remaining_price = (buy_order.price - sell_order.price) * trade_shares;
        sqlx::query(r#"UPDATE wallets SET locked_balance = locked_balance - $1, balance = balance + $2 WHERE user_id = $3"#)
        .bind(total_cost + remaining_price)
        .bind(remaining_price)
        .bind(buy_order.user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE holdings
            SET shares = shares + $1, updated_at = $2
            WHERE user_id = $3 AND market_id = $4 AND outcome_id = $5
            "#,
        )
        .bind(trade_shares)
        .bind(Utc::now())
        .bind(buy_order.user_id)
        .bind(buy_order.market_id)
        .bind(buy_order.outcome_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO transactions (wallet_id, amount, type)
               VALUES ((SELECT id FROM wallets WHERE user_id = $1), $2, $3)"#,
        )
        .bind(buy_order.user_id)
        .bind(total_cost)
        .bind(TransactionType::BUY)
        .execute(&mut *tx)
        .await?;

        // Seller
        sqlx::query(
            r#"
            UPDATE holdings
            SET locked_shares = locked_shares - $1, updated_at = $2
            WHERE user_id = $3 AND market_id = $4 AND outcome_id = $5
            "#,
        )
        .bind(trade_shares)
        .bind(Utc::now())
        .bind(sell_order.user_id)
        .bind(sell_order.market_id)
        .bind(sell_order.outcome_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"UPDATE wallets
               SET balance = balance + $1, updated_at = $2
               WHERE user_id = $3
            "#,
        )
        .bind(total_cost)
        .bind(Utc::now())
        .bind(sell_order.user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(r#"INSERT INTO transactions (wallet_id, amount, type) VALUES ((SELECT id FROM wallets WHERE user_id = $1), $2, $3)"#)
            .bind(sell_order.user_id)
            .bind(total_cost)
            .bind(TransactionType::SELL)
            .execute(&mut *tx)
            .await?;

        sqlx::query(r#"UPDATE outcome SET current_price = $1 WHERE id = $2"#)
            .bind(sell_order.price)
            .bind(sell_order.outcome_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(trade)
    }
}
