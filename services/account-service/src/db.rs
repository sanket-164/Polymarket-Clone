use async_trait::async_trait;
use common::{database::client::PGClient, model::user::User};
use uuid::Uuid;

#[async_trait]
pub trait AccountExt {
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn create_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error>;
}

#[async_trait]
impl AccountExt for PGClient {
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

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, name, email, password, picture, mobile_no, created_at, updated_at FROM users WHERE id = $1"#,
            user_id
        ).fetch_optional(&self.pool).await?;

        Ok(user)
    }

    async fn create_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO users (name, email, password)
            VALUES ($1, $2, $3)
            RETURNING id, name, email, password, picture, mobile_no, created_at, updated_at"#,
            name.into(),
            email.into(),
            password.into()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}
