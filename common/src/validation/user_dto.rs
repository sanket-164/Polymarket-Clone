use core::str;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

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
pub struct DepositBalance {
    #[validate(custom(
        function = "validate_positive_decimal",
        message = "Balance must be greater than zero"
    ))]
    pub balance: Decimal,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawBalance {
    #[validate(custom(
        function = "validate_positive_decimal",
        message = "Balance must be greater than zero"
    ))]
    pub balance: Decimal,
}
