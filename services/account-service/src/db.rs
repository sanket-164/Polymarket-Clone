use async_trait::async_trait;
use common::{
    database::client::PGClient,
    model::user::{User, Wallet},
};
use rust_decimal::Decimal;
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
    async fn update_user<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        name: T,
        email: T,
        mobile_no: Option<T>,
    ) -> Result<User, sqlx::Error>;
    async fn update_user_picture<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        picture: T,
    ) -> Result<User, sqlx::Error>;
    async fn get_balance(&self, user_id: Uuid) -> Result<Wallet, sqlx::Error>;
    async fn deposite_balance(&self, user_id: Uuid, amount: Decimal)
    -> Result<Wallet, sqlx::Error>;
    async fn withdraw_balance(&self, user_id: Uuid, amount: Decimal)
    -> Result<Wallet, sqlx::Error>;
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

        sqlx::query!(
            r#"INSERT INTO wallets (user_id)
            VALUES ($1)"#,
            user.id
        )
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        name: T,
        email: T,
        mobile_no: Option<T>,
    ) -> Result<User, sqlx::Error> {
        let name = name.into();
        let email = email.into();
        let mobile_no = mobile_no.map(|m| m.into());

        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users 
            SET name = $1, email = $2, mobile_no = COALESCE($3, mobile_no)
            WHERE id = $4
            RETURNING id, name, email, password, picture, mobile_no, created_at, updated_at
            "#,
            name,
            email,
            mobile_no,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user_picture<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        picture: T,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users SET picture = $1 WHERE id = $2 
            RETURNING id, name, email, password, picture, mobile_no, created_at, updated_at
            "#,
            picture.into(),
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn get_balance(&self, user_id: Uuid) -> Result<Wallet, sqlx::Error> {
        let wallet = sqlx::query_as!(
            Wallet,
            r#"
            SELECT id, user_id, balance as "balance!: Decimal", locked_balance as "locked_balance!: Decimal", created_at, updated_at
            FROM wallets 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(wallet)
    }

    async fn deposite_balance(
        &self,
        user_id: Uuid,
        amount: Decimal,
    ) -> Result<Wallet, sqlx::Error> {
        let wallet = sqlx::query_as!(
            Wallet,
            r#"
            UPDATE wallets SET balance = balance + $1
            WHERE user_id = $2 
            RETURNING id, user_id, balance as "balance!: Decimal", locked_balance as "locked_balance!: Decimal", created_at, updated_at
            "#,
            amount,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO transactions (wallet_id, amount, type) 
            VALUES ($1, $2, 'DEPOSIT')
            "#,
            wallet.id,
            amount
        )
        .execute(&self.pool)
        .await?;

        Ok(wallet)
    }

    async fn withdraw_balance(
        &self,
        user_id: Uuid,
        amount: Decimal,
    ) -> Result<Wallet, sqlx::Error> {
        let wallet = sqlx::query_as!(
            Wallet,
            r#"
            UPDATE wallets SET balance = balance - $1
            WHERE user_id = $2 
            RETURNING id, user_id, balance as "balance!: Decimal", locked_balance as "locked_balance!: Decimal", created_at, updated_at
            "#,
            amount,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO transactions (wallet_id, amount, type) 
            VALUES ($1, $2, 'WITHDRAW')
            "#,
            wallet.id,
            amount
        )
        .execute(&self.pool)
        .await?;

        Ok(wallet)
    }
}
