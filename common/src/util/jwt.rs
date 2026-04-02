use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::error::{ErrorMessage, HttpError};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn generate_token(
    user_id: &str,
    secret: &[u8],
    expires_in: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    if user_id.is_empty() {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidSubject.into());
    }

    let now = Utc::now();

    let iat = now.timestamp() as usize;

    let exp = (now + Duration::minutes(expires_in as i64)).timestamp() as usize;

    let data = TokenClaims {
        sub: user_id.to_string(),
        iat,
        exp,
    };

    encode(&Header::default(), &data, &EncodingKey::from_secret(secret))
}

pub fn decode_token<T: Into<String>>(token: T, secret: &[u8]) -> Result<String, HttpError> {
    let decode = decode::<TokenClaims>(
        &token.into(),
        &DecodingKey::from_secret(secret),
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    );

    match decode {
        Ok(token) => Ok(token.claims.sub),

        Err(_err) => Err(HttpError::unauthorized(
            ErrorMessage::InvalidToken.to_string(),
        )),
    }
}
