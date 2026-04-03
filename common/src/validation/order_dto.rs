use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::model::market::OrderType;

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
