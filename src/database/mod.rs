use eyre::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};

mod models;

pub struct Database {
    pool: PgPool,
}

impl Database {
    #[cold]
    pub fn new(uri: &str) -> Result<Self> {
        let pool = PgPoolOptions::new().connect_lazy(uri)?;

        Ok(Self { pool })
    }
}
