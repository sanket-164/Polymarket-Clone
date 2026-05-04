use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::model::{TransactionType, User};

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUserDTO {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Provide valid email address")
    )]
    pub email: String,

    #[validate(
        length(min = 1, message = "Password is required"),
        length(min = 8, message = "Password must be at least 8 characters")
    )]
    pub password: String,

    #[validate(
        length(min = 1, message = "Confirm password is required"),
        must_match(other = "password", message = "Passwords do not match")
    )]
    #[serde(rename = "confirmPassword")]
    pub confirm_password: String,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct LoginUserDTO {
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
pub struct ResetPassword {
    #[validate(
        length(min = 1, message = "Old Password is required"),
        length(min = 8, message = "Password must be at least 8 characters")
    )]
    #[serde(rename = "oldPassword")]
    pub old_password: String,

    #[validate(
        length(min = 1, message = "New Password is required"),
        length(min = 8, message = "Password must be at least 8 characters")
    )]
    #[serde(rename = "newPassword")]
    pub new_password: String,

    #[validate(
        length(min = 1, message = "Confirm password is required"),
        must_match(other = "new_password", message = "Passwords do not match")
    )]
    #[serde(rename = "confirmPassword")]
    pub confirm_password: String,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserDTO {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Provide valid email address")
    )]
    pub email: String,

    #[serde(rename = "mobileNo")]
    pub mobile_no: Option<String>,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserPictureDTO {
    #[validate(length(min = 1, message = "Picture is required"))]
    pub picture: String,
}

fn validate_positive_decimal(value: &Decimal) -> Result<(), ValidationError> {
    if *value <= Decimal::ZERO {
        return Err(ValidationError::new("Balance must be greater than zero"));
    }
    Ok(())
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct DepositBalanceDTO {
    #[validate(custom(
        function = "validate_positive_decimal",
        message = "Balance must be greater than zero"
    ))]
    pub balance: Decimal,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawBalanceDTO {
    #[validate(custom(
        function = "validate_positive_decimal",
        message = "Balance must be greater than zero"
    ))]
    pub balance: Decimal,
}

fn validate_order_field(value: &str) -> Result<(), ValidationError> {
    match value {
        "amount" | "created_at" => Ok(()),
        _ => Err(ValidationError::new(
            "Invalid order field. Must be 'amount' or 'created_at'",
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
pub struct TransactionsQueryDTO {
    #[validate(custom(function = "validate_order_field"))]
    pub order_field: Option<String>,

    #[validate(custom(function = "validate_order_by"))]
    pub order_by: Option<String>,

    pub transaction_type: Option<TransactionType>,

    #[validate(range(min = 1))]
    pub limit: Option<i64>,

    #[validate(range(min = 0))]
    pub skip: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub picture: Option<String>,
    pub mobile_no: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
            picture: user.picture,
            mobile_no: user.mobile_no,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
