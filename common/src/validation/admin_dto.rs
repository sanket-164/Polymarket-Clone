use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::model::Admin;

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct LoginAdminDTO {
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Provide valid email address")
    )]
    pub email: String,

    #[validate(
        length(min = 1, message = "Password is required"),
        length(min = 8, message = "Password must be at least 8 charaters")
    )]
    pub password: String,
}

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

fn validate_market_dates(dto: &CreateUpdateMarketDTO) -> Result<(), ValidationError> {
    let now = Utc::now();

    let start_at = dto
        .start_at
        .ok_or_else(|| ValidationError::new("start_at is required"))?;

    let close_at = dto
        .close_at
        .ok_or_else(|| ValidationError::new("close_at is required"))?;

    if start_at <= now {
        return Err(ValidationError::new(
            "start_at must be greater than the current time",
        ));
    }

    if start_at >= close_at {
        return Err(ValidationError::new("start_at must be less than close_at"));
    }

    Ok(())
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
#[validate(schema(function = "validate_market_dates"))]
pub struct CreateUpdateMarketDTO {
    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,

    #[validate(length(min = 1, message = "Description is required"))]
    pub desciption: String,

    #[validate(length(min = 1, message = "Category is required"))]
    pub category: String,

    pub start_at: Option<DateTime<Utc>>,
    pub close_at: Option<DateTime<Utc>>,
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
