//! CLI handlers for `server start` and `server stop`.
//!
//! Run the server in the foreground; to run in the background use the shell (e.g. `server start &`).
//! The PID is written to the PID file so `server stop` can send SIGTERM.

use std::io::Write;

use crate::api::server_start;
use crate::cli::{CliError, ServerStartOpts};
use crate::config::Config;

/// Start the API server (foreground). Write PID to the PID file so `server stop` works when run with `&`.
pub async fn start(
    pool: &sqlx::SqlitePool,
    opts: &ServerStartOpts,
    config_override: Option<&Config>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> Result<(), CliError> {
    let config = match config_override {
        Some(c) => c.clone(),
        None => Config::from_env().map_err(|e| CliError::Other(e.into()))?,
    };
    let config = &config;
    let bind = opts
        .bind
        .as_deref()
        .unwrap_or(config.bind.as_str())
        .to_string();

    run_server(config, pool, &bind, stdout, stderr).await
}

/// Run server in foreground; write PID file so `server stop` can find the process.
async fn run_server(
    config: &Config,
    pool: &sqlx::SqlitePool,
    bind: &str,
    _stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> Result<(), CliError> {
    if let Err(e) = std::fs::write(&config.pid_file, format!("{}\n", std::process::id())) {
        let _ = writeln!(stderr, "failed to write PID file: {e}");
        return Err(CliError::Other(e.into()));
    }
    tracing::info!("starting server on {}", bind);
    if let Err(e) = server_start(config, pool, bind).await {
        let _ = std::fs::remove_file(&config.pid_file);
        let _ = writeln!(stderr, "{e}");
        return Err(CliError::Other(e.into()));
    }
    Ok(())
}

/// Stop the server by sending SIGTERM to the PID in the PID file.
#[allow(unused_variables)]
pub fn stop(
    config_override: Option<&Config>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> Result<(), CliError> {
    let Some(config) = config_override else {
        let c = Config::from_env().map_err(|e| CliError::Other(e.into()))?;
        return stop_with_config(&c, stdout, stderr);
    };
    stop_with_config(config, stdout, stderr)
}

fn stop_with_config(
    config: &Config,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let pid_content = std::fs::read_to_string(&config.pid_file).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CliError::Validation(format!("PID file not found: {}", config.pid_file))
        } else {
            CliError::Other(e.into())
        }
    })?;
    let pid_str = pid_content.trim().trim_end_matches('\n');
    let pid: i32 = pid_str.parse().map_err(|_| {
        CliError::Validation(format!("invalid PID in {}: {}", config.pid_file, pid_str))
    })?;

    #[cfg(unix)]
    {
        use nix::sys::signal::{Signal, kill};
        use nix::unistd::Pid;

        let process_id = Pid::from_raw(pid);
        kill(process_id, Signal::SIGTERM).map_err(|e| {
            CliError::Validation(format!("failed to send SIGTERM to process {pid}: {e}"))
        })?;
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        let _ = writeln!(_stderr, "server stop is not implemented on this platform");
        return Err(CliError::Validation(
            "server stop not supported on this platform".to_string(),
        ));
    }

    let _ = writeln!(stdout, "sent SIGTERM to process {pid}");
    Ok(())
}
