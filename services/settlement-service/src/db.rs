use async_trait::async_trait;
use chrono::Utc;
use common::database::client::PGClient;
use uuid::Uuid;

#[async_trait]
pub trait MarketExt {
    async fn resolve_market(
        &self,
        market_id: Uuid,
        winning_outcome_id: Uuid,
    ) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl MarketExt for PGClient {
    async fn resolve_market(
        &self,
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
            WHERE w.user_id = '11111111-1111-1111-1111-111111111111'",
        )
        .bind(market_id)
        .bind(winning_outcome_id)
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
            WHERE w.user_id = '11111111-1111-1111-1111-111111111111'",
        )
        .bind(Utc::now())
        .bind(market_id)
        .bind(winning_outcome_id)
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
