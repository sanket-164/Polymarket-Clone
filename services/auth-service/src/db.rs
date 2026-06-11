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
}

#[async_trait]
impl AuthExt for PGClient {
    async fn get_admin_by_email(&self, email: &str) -> Result<Option<Admin>, sqlx::Error> {
        let admin = sqlx::query_as!(
            Admin,
            r#"SELECT id, name, email, password, created_at, updated_at FROM admins WHERE email = $1"#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(admin)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, name, email, password, picture, mobile_no, created_at, updated_at FROM users WHERE email = $1"#,
            email
        )
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
}
