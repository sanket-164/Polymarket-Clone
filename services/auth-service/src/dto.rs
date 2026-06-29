use chrono::{DateTime, Utc};
use common::model::User;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

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
pub struct RegisterUserDTO {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Provide valid email address")
    )]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
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

    #[validate(length(min = 8, message = "Password must be at least 8 charaters"))]
    pub password: String,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct ResetPassword {
    #[validate(length(min = 8, message = "Old Password must be at least 8 characters"))]
    #[serde(rename = "oldPassword")]
    pub old_password: String,

    #[validate(length(min = 8, message = "New Password must be at least 8 characters"))]
    #[serde(rename = "newPassword")]
    pub new_password: String,

    #[validate(
        length(min = 1, message = "Confirm password is required"),
        must_match(other = "new_password", message = "Passwords do not match")
    )]
    #[serde(rename = "confirmPassword")]
    pub confirm_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
