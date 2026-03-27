use sqlx::{Pool, Postgres};

#[derive(Debug, Clone)]
pub struct PGClient {
    pub pool: Pool<Postgres>,
}

impl PGClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        PGClient { pool }
    }
}
