use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    database::client::PGClient,
    model::{Admin, Session, User},
};
use uuid::Uuid;

#[async_trait]
pub trait AuthExt {
    async fn get_admin_by_email(&self, email: &str) -> Result<Option<Admin>, sqlx::Error>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn create_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error>;
    async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Session, sqlx::Error>;
    async fn get_session_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, sqlx::Error>;
    async fn reset_password(
        &self,
        email: &str,
        hashed_password: String,
    ) -> Result<Option<User>, sqlx::Error>;
}

#[async_trait]
impl AuthExt for PGClient {
    async fn get_admin_by_email(&self, email: &str) -> Result<Option<Admin>, sqlx::Error> {
        let query = r#"
            SELECT id, name, email, password, created_at, updated_at
            FROM admins
            WHERE email = $1
        "#;

        let admin = sqlx::query_as::<_, Admin>(query)
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(admin)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let query = r#"
            SELECT id, name, email, password, picture, mobile_no, created_at, updated_at
            FROM users
            WHERE email = $1
        "#;

        let user = sqlx::query_as::<_, User>(query)
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn create_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let user_query = r#"
            INSERT INTO users (name, email, password)
            VALUES ($1, $2, $3)
            RETURNING id, name, email, password, picture, mobile_no, created_at, updated_at
        "#;

        let user = sqlx::query_as::<_, User>(user_query)
            .bind(name.into())
            .bind(email.into())
            .bind(password.into())
            .fetch_one(&mut *tx)
            .await?;

        let wallet_query = r#"
            INSERT INTO wallets (user_id)
            VALUES ($1)
        "#;

        sqlx::query(wallet_query)
            .bind(user.id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(user)
    }

    async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Session, sqlx::Error> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, token_hash, expires_at, revoked_at, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    async fn get_session_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, sqlx::Error> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, user_id, token_hash, expires_at, revoked_at, created_at, updated_at
            FROM sessions
            WHERE token_hash = $1
        "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    async fn reset_password(
        &self,
        email: &str,
        hashed_password: String,
    ) -> Result<Option<User>, sqlx::Error> {
        let query = r#"
            UPDATE users
            SET password = $1, updated_at = CURRENT_TIMESTAMP
            WHERE email = $2 AND deleted_at IS NULL
            RETURNING id, name, email, password, picture, mobile_no, created_at, updated_at
        "#;

        let user = sqlx::query_as::<_, User>(query)
            .bind(hashed_password)
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }
}
