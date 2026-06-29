use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use common::model::MarketStatus;

fn validate_market_order_field(value: &str) -> Result<(), ValidationError> {
    match value {
        "start_at" | "close_at" | "created_at" => Ok(()),
        _ => Err(ValidationError::new(
            "Invalid order field. Must be 'start_at', 'close_at' or 'created_at'",
        )),
    }
}

fn validate_order_by(value: &str) -> Result<(), ValidationError> {
    match value {
        "ASC" | "DESC" => Ok(()),
        _ => Err(ValidationError::new(
            "Invalid order direction. Must be 'ASC' or 'DESC'",
        )),
    }
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct MarketQueryDTO {
    #[validate(custom(function = "validate_market_order_field"))]
    pub order_field: Option<String>,

    #[validate(custom(function = "validate_order_by"))]
    pub order_by: Option<String>,

    pub status: Option<MarketStatus>,
    pub category: Option<String>,
    pub start_after: Option<DateTime<Utc>>,
    pub start_before: Option<DateTime<Utc>>,
    pub close_after: Option<DateTime<Utc>>,
    pub close_before: Option<DateTime<Utc>>,

    #[validate(range(min = 1))]
    pub limit: Option<i64>,

    #[validate(range(min = 0))]
    pub skip: Option<i64>,
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
