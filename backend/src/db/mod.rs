//! Database setup: connection pool and migrations.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

pub mod category;
pub mod product;
pub mod user;

/// Errors that can occur during database operations.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Returned when a connection or query fails.
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Returned when a migration fails.
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    /// Returned when a row cannot be mapped to a domain type (e.g. invalid UUID or validation failed).
    #[error("invalid data: {0}")]
    InvalidData(String),
}

/// Create a `SQLite` connection pool.
///
/// Creates the database file if it does not exist.
///
/// # Errors
///
/// Returns [`DbError::Sqlx`] if the connection cannot be established.
pub async fn create_pool(database_path: &str) -> Result<SqlitePool, DbError> {
    let options = SqliteConnectOptions::new()
        .filename(database_path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new().connect_with(options).await?;

    Ok(pool)
}

/// Run all pending migrations against the given pool.
///
/// # Errors
///
/// Returns [`DbError::Migrate`] if any migration fails.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
