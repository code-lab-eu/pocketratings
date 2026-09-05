//! Helpers shared by the integration tests in this directory.

use pocketratings::config::Config;

/// Build a configuration for tests, with fixed values for everything but the database path.
///
/// Adding a [`Config`] field only touches this constructor. Override a single field with struct
/// update syntax, e.g. `Config { pid_file, ..test_config(path) }`.
pub fn test_config(database_path: &str) -> Config {
    Config {
        database_path: database_path.to_string(),
        jwt_secret: "test-secret".to_string(),
        jwt_expiration_seconds: 3600,
        jwt_refresh_threshold_seconds: 600,
        bind: "127.0.0.1:0".to_string(),
        pid_file: std::env::temp_dir()
            .join("pocketratings-test.pid")
            .to_string_lossy()
            .into_owned(),
    }
}
