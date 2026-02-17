//! Application configuration loaded from environment variables.

use std::env;

/// Default JWT expiration: 30 days in seconds.
const DEFAULT_JWT_EXPIRATION_SECONDS: u64 = 30 * 24 * 3600;

/// Default refresh threshold: if token expires within this many seconds, issue a new one (7 days).
const DEFAULT_JWT_REFRESH_THRESHOLD_SECONDS: u64 = 7 * 24 * 3600;

/// Application configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the `SQLite` database file.
    pub database_path: String,

    /// Secret used to sign and verify JWT tokens.
    pub jwt_secret: String,

    /// Token expiration in seconds (default: 30 days).
    pub jwt_expiration_seconds: u64,

    /// If token expires within this many seconds, issue a new one in X-New-Token (default: 7 days).
    pub jwt_refresh_threshold_seconds: u64,

    /// Address the API server binds to (e.g. `127.0.0.1:3099`).
    pub bind: String,

    /// Path to the PID file for daemon mode (e.g. `/tmp/pocketratings.pid` on Unix).
    pub pid_file: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// - `DB_PATH` — database path (default: `./pocketratings.db`)
    /// - `JWT_SECRET` — JWT signing secret (**required**)
    /// - `JWT_EXPIRATION_SECONDS` — token expiration in seconds (default: 30 days)
    /// - `JWT_REFRESH_THRESHOLD_SECONDS` — issue new token if exp within this (default: 7 days)
    /// - `BIND` — server bind address (default: `127.0.0.1:3099`)
    /// - `PID_FILE` — path to PID file for daemon mode (default: temp dir + `pocketratings.pid`)
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Missing`] if a required variable is not set.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_path =
            env::var("DB_PATH").unwrap_or_else(|_| String::from("./pocketratings.db"));

        let jwt_secret = env::var("JWT_SECRET").map_err(|_| ConfigError::Missing("JWT_SECRET"))?;

        let bind = env::var("BIND").unwrap_or_else(|_| String::from("127.0.0.1:3099"));

        let jwt_expiration_seconds = env::var("JWT_EXPIRATION_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_JWT_EXPIRATION_SECONDS);

        let jwt_refresh_threshold_seconds = env::var("JWT_REFRESH_THRESHOLD_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_JWT_REFRESH_THRESHOLD_SECONDS);

        let pid_file = env::var("PID_FILE").unwrap_or_else(|_| {
            env::temp_dir()
                .join("pocketratings.pid")
                .to_string_lossy()
                .into_owned()
        });

        Ok(Self {
            database_path,
            jwt_secret,
            jwt_expiration_seconds,
            jwt_refresh_threshold_seconds,
            bind,
            pid_file,
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
