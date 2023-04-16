use sqlx::postgres::{PgPool, PgPoolOptions};
use std::error::Error;

pub async fn create_pool() -> Result<PgPool, Box<dyn Error>> {
    let db =
        std::env::var("DATABASE_URL").expect("Expected database connection url in environment");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db.as_str())
        .await
        .expect("Database connection to be established");

    Ok(pool)
}
