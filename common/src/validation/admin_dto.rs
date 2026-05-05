use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
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

    #[validate(length(min = 8, message = "Password must be at least 8 charaters"))]
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
trait MarketDates {
    fn start_at(&self) -> Option<DateTime<Utc>>;
    fn close_at(&self) -> Option<DateTime<Utc>>;
}

impl MarketDates for &&CreateMarketDTO {
    fn start_at(&self) -> Option<DateTime<Utc>> {
        self.start_at
    }
    fn close_at(&self) -> Option<DateTime<Utc>> {
        self.close_at
    }
}

impl MarketDates for &&UpdateMarketDTO {
    fn start_at(&self) -> Option<DateTime<Utc>> {
        self.start_at
    }
    fn close_at(&self) -> Option<DateTime<Utc>> {
        self.close_at
    }
}

fn validate_market_dates(dto: impl MarketDates) -> Result<(), ValidationError> {
    let now = Utc::now();

    let start_at = dto
        .start_at()
        .ok_or_else(|| ValidationError::new("start_at is required"))?;

    let close_at = dto
        .close_at()
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

fn validate_start_price(value: &Decimal) -> Result<(), ValidationError> {
    if *value <= Decimal::ZERO || *value >= Decimal::ONE {
        return Err(ValidationError::new("Start price must be between 0 and 1"));
    }
    Ok(())
}

fn validate_total_shares(value: &Decimal) -> Result<(), ValidationError> {
    if *value <= Decimal::ONE_HUNDRED {
        return Err(ValidationError::new(
            "Total Shares must be greater than 100",
        ));
    }
    Ok(())
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct CreateOutcomeDTO {
    #[validate(length(min = 1, message = "Label is required"))]
    pub label: String,

    #[validate(custom(
        function = "validate_start_price",
        message = "Start price must be between 0 and 1"
    ))]
    pub start_price: Decimal,

    #[validate(custom(
        function = "validate_total_shares",
        message = "Total Shares must be greater than 100"
    ))]
    pub total_shares: Decimal,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
#[validate(schema(function = "validate_market_dates"))]
pub struct CreateMarketDTO {
    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,

    #[validate(length(min = 1, message = "Description is required"))]
    pub desciption: String,

    #[validate(length(min = 1, message = "Category is required"))]
    pub category: String,

    pub start_at: Option<DateTime<Utc>>,
    pub close_at: Option<DateTime<Utc>>,

    #[validate(nested)]
    pub first_outcome: CreateOutcomeDTO,

    #[validate(nested)]
    pub second_outcome: CreateOutcomeDTO,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOutcomeDTO {
    pub outcome_id: Uuid,
    #[validate(length(min = 1, message = "Label is required"))]
    pub label: String,

    #[validate(custom(
        function = "validate_start_price",
        message = "Start price must be between 0 and 1"
    ))]
    pub start_price: Decimal,

    #[validate(custom(
        function = "validate_total_shares",
        message = "Total Shares must be greater than 100"
    ))]
    pub total_shares: Decimal,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
#[validate(schema(function = "validate_market_dates"))]
pub struct UpdateMarketDTO {
    pub market_id: Uuid,
    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,

    #[validate(length(min = 1, message = "Description is required"))]
    pub desciption: String,

    #[validate(length(min = 1, message = "Category is required"))]
    pub category: String,

    pub start_at: Option<DateTime<Utc>>,
    pub close_at: Option<DateTime<Utc>>,

    #[validate(nested)]
    pub first_outcome: UpdateOutcomeDTO,

    #[validate(nested)]
    pub second_outcome: UpdateOutcomeDTO,
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
