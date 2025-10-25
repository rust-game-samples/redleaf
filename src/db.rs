use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::env;

pub type DbPool = SqlitePool;

pub async fn init_db() -> anyhow::Result<DbPool> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:redleaf.db".to_string());

    tracing::info!("Connecting to database: {}", database_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
