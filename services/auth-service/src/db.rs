use async_trait::async_trait;
use common::{
    database::client::PGClient,
    model::{Admin, User},
};

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
