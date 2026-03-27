use dotenv::dotenv;

#[derive(Debug, Clone)]
pub struct JWTConfig {
    pub jwt_secret_key: String,
    pub jwt_expiration_time: u64,
}

impl JWTConfig {
    pub fn init() -> Self {
        dotenv().ok();

        let jwt_secret_key =
            std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY is not set in .env file.");

        let jwt_expiration_time = std::env::var("JWT_EXPIRATION_TIME")
            .expect("JWT_EXPIRATION_TIME is not set in .env file.")
            .parse::<u64>()
            .expect("JWT_EXPIRATION_TIME must be a valid integer");

        JWTConfig {
            jwt_secret_key,
            jwt_expiration_time,
        }
    }
}
