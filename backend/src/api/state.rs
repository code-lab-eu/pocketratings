//! Shared state for the API (config and database pool).

use sqlx::SqlitePool;

use crate::config::Config;

/// Application state injected into all routes that need config or database.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration (env-loaded).
    pub config: Config,
    /// Database connection pool.
    pub pool: SqlitePool,
}
