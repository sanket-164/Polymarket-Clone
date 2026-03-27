use dotenv::dotenv;

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
            database_url: database_url,
            pool_size_each_service,
        }
    }
}
