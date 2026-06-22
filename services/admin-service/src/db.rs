use async_trait::async_trait;
use chrono::Utc;
use common::{database::client::PGClient, model::Admin};
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
