use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

use common::model::{OrderSide, OrderStatus};

fn validate_positive_decimal(value: &Decimal) -> Result<(), ValidationError> {
    if *value <= Decimal::ZERO {
        return Err(ValidationError::new("must_be_positive"));
    }
    Ok(())
}

fn validate_expires_at(expires_at: &DateTime<Utc>) -> Result<(), ValidationError> {
    if expires_at.to_utc() <= Utc::now() {
        return Err(ValidationError::new("must_be_in_future"));
    }
    Ok(())
}

// ---------- LIMIT ----------

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct LimitOrderDTO {
    pub market_id: Uuid,
    pub outcome_id: Uuid,
    pub side: OrderSide,

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

    #[validate(custom(
        function = "validate_expires_at",
        message = "expires_at must be greater than the current time"
    ))]
    pub expires_at: DateTime<Utc>,
}

// ---------- MARKET ----------s

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrderDTO {
    pub market_id: Uuid,
    pub outcome_id: Uuid,
    pub side: OrderSide,
    pub shares: Option<Decimal>,
    pub quote_amount: Option<Decimal>,
    // no price, no expires_at — market orders fill immediately against the book
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "order_type", rename_all = "lowercase")]
pub enum PlaceOrderDTO {
    Limit(LimitOrderDTO),
    Market(MarketOrderDTO),
}

// validator's derive macro doesn't support enums directly (as of validator 0.18),
// so Validate is implemented by hand, delegating to whichever variant is present.
impl Validate for PlaceOrderDTO {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            PlaceOrderDTO::Limit(dto) => dto.validate(),
            PlaceOrderDTO::Market(dto) => dto.validate(),
        }
    }
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
    pub side: Option<OrderSide>,
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
