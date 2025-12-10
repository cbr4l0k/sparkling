use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;
use crate::infrastructure::config::AppConfig;

/// Create a SQLite connection pool from configuration
pub async fn create_pool(config: &AppConfig) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .connect(&config.database.connection_string())
        .await?;

    // Test the connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await?;

    tracing::info!("Database connection pool established");

    Ok(pool)
}
