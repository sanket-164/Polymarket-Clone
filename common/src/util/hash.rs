use crate::error::ErrorMessage;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

pub fn generate(password: impl Into<String>) -> Result<String, ErrorMessage> {
    let password = password.into();

    let salt = SaltString::generate(&mut OsRng);

    let hash_password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| ErrorMessage::HashingError)?
        .to_string();

    Ok(hash_password)
}

pub fn compare(password: &str, hashed_password: &str) -> Result<bool, ErrorMessage> {
    let parsed_hash = PasswordHash::new(hashed_password).map_err(|_| ErrorMessage::HashingError)?;

    let password_matched = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    Ok(password_matched)
}
