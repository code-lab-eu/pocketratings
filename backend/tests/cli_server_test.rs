//! Integration tests for `pocketratings server start` and `server stop` CLI.

use std::io::Cursor;
use std::process::Stdio;
use std::sync::Arc;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use pocketratings::cli;
use pocketratings::config::Config;
use serial_test::serial;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, Notify};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn run_server_stop(
    config_override: Option<&Config>,
) -> (Result<(), cli::CliError>, String, String) {
    let args: Vec<std::ffi::OsString> =
        vec!["pocketratings".into(), "server".into(), "stop".into()];

    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let result = cli::run(
        args.into_iter(),
        None,
        config_override,
        &mut stdout,
        &mut stderr,
    )
    .await;
    let stdout_str = String::from_utf8(stdout.into_inner()).expect("stdout UTF-8");
    let stderr_str = String::from_utf8(stderr.into_inner()).expect("stderr UTF-8");
    (result, stdout_str, stderr_str)
}

struct TestServer {
    _dir: tempfile::TempDir,
    server: tokio::process::Child,
    #[allow(dead_code)] // kept for potential log assertions (e.g. healthmonitor-style)
    stderr: Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(server_id) = self.server.id() {
            tokio::task::spawn_blocking(move || {
                #[allow(clippy::cast_possible_wrap)]
                let pid = Pid::from_raw(server_id as i32);
                let _ = kill(pid, Signal::SIGINT);
            });
        }
    }
}

impl TestServer {
    async fn start() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("server_test.db");

        let mut server = tokio::process::Command::new("cargo")
            .args(["run", "--", "server", "start"])
            .env("DB_PATH", db_path.to_str().expect("path"))
            .env("JWT_SECRET", "test-secret")
            .env("BIND", "127.0.0.1:0")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("cargo run should spawn a child process");

        let stderr = server.stderr.take().expect("stderr should be captured");
        let stderr_lines = Arc::new(Mutex::new(BufReader::new(stderr).lines()));

        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();
        let lines_clone = Arc::clone(&stderr_lines);
        tokio::spawn(async move {
            while let Ok(Some(line)) = lines_clone.lock().await.next_line().await {
                if line.contains("listening on") {
                    notify_clone.notify_one();
                    break;
                }
            }
        });
        notify.notified().await;

        Self {
            _dir: dir,
            server,
            stderr: stderr_lines,
        }
    }

    async fn stop(&mut self) {
        #[allow(clippy::cast_possible_wrap)]
        let pid = Pid::from_raw(self.server.id().expect("server process should be running") as i32);
        kill(pid, Signal::SIGINT).expect("SIGINT should be sent");

        // Wait for the process (cargo, which we sent SIGINT) to exit.
        self.server
            .wait()
            .await
            .expect("server process should exit");
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Server start without pool returns error and message about database pool required.
#[tokio::test]
async fn server_start_cli_requires_database() {
    let args: Vec<std::ffi::OsString> =
        vec!["pocketratings".into(), "server".into(), "start".into()];
    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let result = cli::run(args.into_iter(), None, None, &mut stdout, &mut stderr).await;
    assert!(result.is_err(), "server start without pool should fail");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("database pool required"),
        "error should mention database pool: {err}",
    );
}

/// Server stop with nonexistent PID file returns error and message about PID file.
#[tokio::test]
async fn server_stop_no_pid_file_returns_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let pid_path = dir.path().join("nonexistent.pid");
    let config = Config {
        database_path: "./pocketratings.db".to_string(),
        jwt_secret: "test".to_string(),
        jwt_expiration_seconds: 3600,
        jwt_refresh_threshold_seconds: 600,
        bind: "127.0.0.1:3099".to_string(),
        pid_file: pid_path.to_string_lossy().into_owned(),
    };

    let (result, _stdout, _stderr) = run_server_stop(Some(&config)).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("PID file not found") || err_msg.contains("not found"),
        "error should mention PID file or not found: {err_msg}",
    );
}

/// Start the server via the CLI binary, then stop it with SIGINT.
#[tokio::test]
#[serial]
async fn server_start_and_stop_via_cli() {
    let mut server = TestServer::start().await;
    server.stop().await;
}
