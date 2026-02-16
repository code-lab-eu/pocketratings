//! Application configuration loaded from environment variables.

use std::env;

/// Application configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the `SQLite` database file.
    pub database_path: String,

    /// Secret used to sign and verify JWT tokens.
    pub jwt_secret: String,

    /// Address the API server binds to (e.g. `127.0.0.1:3099`).
    pub bind: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// - `DB_PATH` — database path (default: `./pocketratings.db`)
    /// - `JWT_SECRET` — JWT signing secret (**required**)
    /// - `BIND` — server bind address (default: `127.0.0.1:3099`)
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Missing`] if a required variable is not set.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_path =
            env::var("DB_PATH").unwrap_or_else(|_| String::from("./pocketratings.db"));

        let jwt_secret = env::var("JWT_SECRET").map_err(|_| ConfigError::Missing("JWT_SECRET"))?;

        let bind = env::var("BIND").unwrap_or_else(|_| String::from("127.0.0.1:3099"));

        Ok(Self {
            database_path,
            jwt_secret,
            bind,
        })
    }
}

/// Errors that can occur when loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A required environment variable is not set.
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
}
