//! Application configuration loaded from environment variables.

use std::env;

/// Application configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the SQLite database file.
    pub database_path: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// - `DB_PATH` — database path (default: `./pocketratings.db`)
    pub fn from_env() -> Self {
        let database_path = env::var("DB_PATH")
            .unwrap_or_else(|_| "./pocketratings.db".to_string());

        Self { database_path }
    }
}
