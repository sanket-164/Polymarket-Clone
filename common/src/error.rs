use core::fmt;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

#[derive(Debug, PartialEq)]
pub enum ErrorMessage {
    WrongCredentials,
    OtpOrPasswordRequired,
    InvalidOtp,
    InvalidToken,
    EmailExist,
    TokenNotGiven,
    HashingError,
    UserNotFound,
    InsufficientBalance,
    MarketNotFound,
    MarketIsNotActive,
    MarketIsNotClosed,
    OutcomeNotFound,
    InsufficientShares,
    CannotPublishOrder,
}

impl ErrorMessage {
    fn to_str(&self) -> String {
        match self {
            ErrorMessage::WrongCredentials => "Wrong credentials are given".to_string(),
            ErrorMessage::OtpOrPasswordRequired => {
                "Either old password or OTP is required".to_string()
            }
            ErrorMessage::InvalidOtp => "Invalid OTP".to_string(),
            ErrorMessage::EmailExist => "Email already exist".to_string(),
            ErrorMessage::InvalidToken => "Token is invalid".to_string(),
            ErrorMessage::TokenNotGiven => "Token is not given".to_string(),
            ErrorMessage::HashingError => "Error while hasing the password".to_string(),
            ErrorMessage::UserNotFound => "User does not exist".to_string(),
            ErrorMessage::InsufficientBalance => "Insufficient Balance".to_string(),
            ErrorMessage::MarketNotFound => "Market does not exist".to_string(),
            ErrorMessage::MarketIsNotActive => "Market is not active".to_string(),
            ErrorMessage::MarketIsNotClosed => "Market is not closed".to_string(),
            ErrorMessage::OutcomeNotFound => "Outcome does not exist".to_string(),
            ErrorMessage::InsufficientShares => "Insufficient Shares".to_string(),
            ErrorMessage::CannotPublishOrder => "Failed to publish order".to_string(),
        }
    }
}

impl fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug, Clone)]
pub struct HttpError {
    pub message: String,
    pub status: StatusCode,
}

impl HttpError {
    pub fn server_error(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: StatusCode::BAD_REQUEST,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: StatusCode::CONFLICT,
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: StatusCode::FORBIDDEN,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: StatusCode::NOT_FOUND,
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: StatusCode::UNAUTHORIZED,
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HttpError: message: {}, status: {}",
            self.message, self.status
        )
    }
}

impl std::error::Error for HttpError {}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let json_response = Json(ErrorResponse {
            status: "fail".to_string(),
            message: self.message.clone(),
        });

        (self.status, json_response).into_response()
    }
}
