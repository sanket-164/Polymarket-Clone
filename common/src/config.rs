use dotenv::dotenv;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub allow_origin: String,
    pub environment: String,
}

impl ServerConfig {
    pub fn init() -> Self {
        dotenv().ok();

        let allow_origin =
            std::env::var("ALLOW_ORIGIN").expect("ALLOW_ORIGIN is not set in .env file");

        let environment =
            std::env::var("ENVIRONMENT").expect("ENVIRONMENT is not set in .env file");

        ServerConfig {
            allow_origin,
            environment,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PGConfig {
    pub database_url: String,
    pub pool_size_each_service: u32,
}

impl PGConfig {
    pub fn init() -> Self {
        dotenv().ok();

        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

        let pool_size_each_service = std::env::var("POOL_SIZE_EACH_SERVICE")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<u32>()
            .expect("POOL_SIZE_EACH_SERVICE must be a valid integer");

        PGConfig {
            database_url,
            pool_size_each_service,
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub redis_url: String,
}

impl RedisConfig {
    pub fn init() -> Self {
        dotenv().ok();

        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL is not set in .env file.");

        RedisConfig { redis_url }
    }
}

#[derive(Debug, Clone)]
pub struct NatsConfig {
    pub nats_url: String,
}

impl NatsConfig {
    pub fn init() -> Self {
        dotenv().ok();

        let nats_url = std::env::var("NATS_URL").expect("NATS_URL is not set in .env file.");

        NatsConfig { nats_url }
    }
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
}

impl SmtpConfig {
    pub fn init() -> Self {
        dotenv().ok();

        let smtp_host = std::env::var("SMTP_HOST").expect("SMTP_HOST is not set in .env file.");
        let smtp_username =
            std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME is not set in .env file.");
        let smtp_password =
            std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD is not set in .env file.");

        SmtpConfig {
            smtp_host,
            smtp_username,
            smtp_password,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RedpandaConfig {
    pub bootstrap_servers: String,
}

impl RedpandaConfig {
    pub fn init() -> Self {
        dotenv().ok();

        let bootstrap_servers =
            std::env::var("BOOTSTRAP_SERVERS").expect("BOOTSTRAP_SERVERS is not set in .env file.");

        RedpandaConfig { bootstrap_servers }
    }
}
