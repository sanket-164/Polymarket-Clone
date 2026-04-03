use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::model::market::{OrderStatus, OrderType};

fn validate_positive_decimal(value: &Decimal) -> Result<(), ValidationError> {
    if *value <= Decimal::ZERO {
        return Err(ValidationError::new("Balance must be greater than zero"));
    }
    Ok(())
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOrderDTO {
    pub market_id: Uuid,
    pub outcome_id: Uuid,
    #[validate(custom(
        function = "validate_positive_decimal",
        message = "Shares must be greater than zero"
    ))]
    pub shares: Decimal,
    #[validate(custom(
        function = "validate_positive_decimal",
        message = "Price must be greater than zero"
    ))]
    pub price: Decimal,
    pub order_type: OrderType,
}

fn validate_after(after: &DateTime<Utc>) -> Result<(), ValidationError> {
    if after >= &Utc::now() {
        return Err(ValidationError::new("after must be less than current time"));
    }
    Ok(())
}

fn validate_order_field(value: &str) -> Result<(), ValidationError> {
    match value {
        "shares" | "price" | "created_at" => Ok(()),
        _ => Err(ValidationError::new(
            "Invalid order field. Must be 'shares', 'price' or 'created_at'",
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

fn validate_before_after(dto: &OrderQueryDTO) -> Result<(), ValidationError> {
    if let (Some(before), Some(after)) = (dto.before, dto.after) {
        if before <= after {
            return Err(ValidationError::new("before must be greater than after"));
        }
    }
    Ok(())
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
#[validate(schema(function = "validate_before_after"))]
pub struct OrderQueryDTO {
    pub market_id: Option<Uuid>,
    pub order_type: Option<OrderType>,
    pub status: Option<OrderStatus>,
    pub before: Option<DateTime<Utc>>,
    #[validate(custom(function = "validate_after"))]
    pub after: Option<DateTime<Utc>>,
    #[validate(custom(function = "validate_order_field"))]
    pub order_field: Option<String>,
    #[validate(custom(function = "validate_order_by"))]
    pub order_by: Option<String>,
    pub limit: Option<i64>,
    pub skip: Option<i64>,
}
