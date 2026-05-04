use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod market;
pub mod user;

pub struct Admin {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
