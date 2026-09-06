//! Integration tests for `pocketratings user reset-link` CLI.

use std::io::Cursor;

use pocketratings::auth::reset_token::{self, TOKEN_TTL_SECONDS};
use pocketratings::cli;
use pocketratings::config::Config;
use pocketratings::db;
use sqlx::Row;

mod common;

use common::test_config;

/// Base URL that [`test_config`] puts in the configuration; the printed link must start with it.
const BASE_URL: &str = "http://localhost:5173";

async fn run_register(pool: &sqlx::SqlitePool, email: &str) -> Result<(), cli::CliError> {
    let args = [
        "pocketratings",
        "user",
        "register",
        "--name",
        "Alice",
        "--email",
        email,
        "--password",
        "oldpass",
    ]
    .into_iter()
    .map(std::ffi::OsString::from);

    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    cli::run(args, Some(pool), None, &mut stdout, &mut stderr).await
}

async fn run_reset_link(
    pool: &sqlx::SqlitePool,
    config: &Config,
    email: &str,
    extra_args: &[&str],
) -> (Result<(), cli::CliError>, String, String) {
    let args = ["pocketratings", "user", "reset-link", "--email", email]
        .into_iter()
        .chain(extra_args.iter().copied())
        .map(std::ffi::OsString::from);

    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let result = cli::run(args, Some(pool), Some(config), &mut stdout, &mut stderr).await;
    let stdout_str = String::from_utf8(stdout.into_inner()).expect("stdout UTF-8");
    let stderr_str = String::from_utf8(stderr.into_inner()).expect("stderr UTF-8");
    (result, stdout_str, stderr_str)
}

/// Create a migrated temp-file pool plus a matching config. Keep the `TempDir` alive.
async fn test_pool(name: &str) -> (sqlx::SqlitePool, Config, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join(format!("{name}.db"));
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");
    let config = test_config(db_path_str);
    (pool, config, dir)
}

/// Extract the `token` query parameter from a `/reset-password?token=...` URL.
fn token_from_url(url: &str) -> String {
    let (_, token) = url
        .split_once("?token=")
        .unwrap_or_else(|| panic!("URL should carry a token query param: {url}"));
    token.trim().to_string()
}

#[tokio::test]
async fn reset_link_prints_url_and_stores_the_token_hash() {
    let (pool, config, _dir) = test_pool("cli_reset_link").await;
    run_register(&pool, "alice@example.com")
        .await
        .expect("register");
    let before = chrono::Utc::now().timestamp();

    let (result, stdout, stderr) = run_reset_link(&pool, &config, "alice@example.com", &[]).await;

    assert!(result.is_ok(), "expected Ok, stderr: {stderr}");
    let mut lines = stdout.lines();
    let url = lines.next().expect("first line is the URL");
    assert!(
        url.starts_with(&format!("{BASE_URL}/reset-password?token=")),
        "unexpected URL: {url}"
    );
    let token = token_from_url(url);
    assert_eq!(
        token.len(),
        64,
        "token should be 64 hex characters: {token}"
    );
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    let expiry_line = lines.next().expect("second line is the expiry");
    assert!(
        expiry_line.contains("Valid for 12 hours"),
        "unexpected expiry line: {expiry_line}"
    );

    let expires_at: i64 =
        sqlx::query("SELECT expires_at FROM password_reset_tokens WHERE token_hash = ?")
            .bind(reset_token::hash(&token))
            .fetch_one(&pool)
            .await
            .expect("token hash should be stored")
            .get("expires_at");
    assert!(
        (before + TOKEN_TTL_SECONDS..=chrono::Utc::now().timestamp() + TOKEN_TTL_SECONDS)
            .contains(&expires_at),
        "expiry should be about 12 hours from now: {expires_at}"
    );
}

#[tokio::test]
async fn reset_link_json_output_has_url_and_expires_at() {
    let (pool, config, _dir) = test_pool("cli_reset_link_json").await;
    run_register(&pool, "alice@example.com")
        .await
        .expect("register");
    let before = chrono::Utc::now().timestamp();

    let (result, stdout, stderr) =
        run_reset_link(&pool, &config, "alice@example.com", &["--output", "json"]).await;

    assert!(result.is_ok(), "expected Ok, stderr: {stderr}");
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("stdout is JSON");
    let url = json["url"].as_str().expect("url is a string");
    assert!(
        url.starts_with(&format!("{BASE_URL}/reset-password?token=")),
        "unexpected URL: {url}"
    );
    let expires_at = json["expires_at"].as_i64().expect("expires_at is a number");
    assert!(expires_at >= before + TOKEN_TTL_SECONDS);

    let stored: i64 =
        sqlx::query("SELECT expires_at FROM password_reset_tokens WHERE token_hash = ?")
            .bind(reset_token::hash(&token_from_url(url)))
            .fetch_one(&pool)
            .await
            .expect("token hash should be stored")
            .get("expires_at");
    assert_eq!(stored, expires_at);
}

#[tokio::test]
async fn reset_link_unknown_email_returns_error() {
    let (pool, config, _dir) = test_pool("cli_reset_link_unknown").await;

    let (result, _stdout, _stderr) =
        run_reset_link(&pool, &config, "nobody@example.com", &[]).await;

    let err = result.expect_err("unknown email should fail");
    assert!(
        err.to_string().contains("not found"),
        "error should mention not found: {err}"
    );
    let count: i64 = sqlx::query("SELECT COUNT(*) AS n FROM password_reset_tokens")
        .fetch_one(&pool)
        .await
        .expect("count tokens")
        .get("n");
    assert_eq!(count, 0, "no token should be stored for an unknown user");
}
