use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use common::model::Admin;

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAdminDTO {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Provide valid email address")
    )]
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Admin> for AdminResponse {
    fn from(admin: Admin) -> Self {
        AdminResponse {
            id: admin.id,
            name: admin.name,
            email: admin.email,
            created_at: admin.created_at,
            updated_at: admin.updated_at,
        }
    }
}
