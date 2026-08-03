use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
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

fn validate_market_dates_create(dto: &CreateMarketDTO) -> Result<(), ValidationError> {
    let now = Utc::now();

    if dto.start_at <= now {
        return Err(ValidationError::new(
            "start_at must be greater than the current time",
        ));
    }

    if dto.start_at >= dto.close_at {
        return Err(ValidationError::new("start_at must be less than close_at"));
    }

    Ok(())
}

fn validate_market_dates_update(dto: &UpdateMarketDTO) -> Result<(), ValidationError> {
    let now = Utc::now();

    if let Some(start_at) = dto.start_at {
        if start_at <= now {
            return Err(ValidationError::new(
                "start_at must be greater than the current time",
            ));
        }
    }

    if let (Some(start_at), Some(close_at)) = (dto.start_at, dto.close_at) {
        if start_at >= close_at {
            return Err(ValidationError::new("start_at must be less than close_at"));
        }
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
#[validate(schema(function = "validate_market_dates_create"))]
pub struct CreateMarketDTO {
    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,

    #[validate(length(min = 1, message = "Description is required"))]
    pub desciption: String,

    #[validate(length(min = 1, message = "Category is required"))]
    pub category: String,

    pub start_at: DateTime<Utc>,
    pub close_at: DateTime<Utc>,

    #[validate(nested)]
    pub first_outcome: CreateOutcomeDTO,

    #[validate(nested)]
    pub second_outcome: CreateOutcomeDTO,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
#[validate(schema(function = "validate_market_dates_update"))]
pub struct UpdateMarketDTO {
    #[validate(length(min = 1, message = "Title is required"))]
    pub title: Option<String>,

    #[validate(length(min = 1, message = "Description is required"))]
    pub desciption: Option<String>,

    #[validate(length(min = 1, message = "Category is required"))]
    pub category: Option<String>,

    pub start_at: Option<DateTime<Utc>>,
    pub close_at: Option<DateTime<Utc>>,
}
