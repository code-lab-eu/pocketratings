//! Database subcommands (backup).

use std::io::Write;

use sqlx::SqlitePool;

use crate::cli::CliError;
use crate::config::Config;
use crate::db;

/// Create a consistent backup of the database while the server may be running.
///
/// Uses `SQLite`'s `VACUUM INTO` to produce a snapshot without stopping the server.
/// The backup is written to the given path (default: `DB_PATH` with `.backup` suffix).
pub async fn backup(
    pool: &SqlitePool,
    config_override: Option<&Config>,
    output: Option<&str>,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let config = match config_override {
        Some(c) => c.clone(),
        None => Config::from_env().map_err(|e| CliError::Other(e.into()))?,
    };
    let output_path = match output {
        Some(p) => p.to_string(),
        None => format!("{}.backup", config.database_path),
    };

    if !is_safe_backup_path(&output_path) {
        return Err(CliError::Validation(format!(
            "invalid backup path (refusing path with '..' or control characters): {output_path}",
        )));
    }

    db::vacuum_into(pool, &output_path).await?;

    writeln!(stdout, "backup written to {output_path}").map_err(|e| CliError::Other(e.into()))?;
    Ok(())
}

/// Reject paths that could escape the intended directory or inject SQL.
fn is_safe_backup_path(path: &str) -> bool {
    !path.contains("..") && !path.contains(|c: char| c.is_control() || c == '\0')
}
