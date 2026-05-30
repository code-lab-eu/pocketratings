//! Integration tests for `pocketratings user set-password` CLI.

use std::io::Cursor;

use pocketratings::cli;
use pocketratings::db;

async fn run_register(
    pool: &sqlx::SqlitePool,
    name: &str,
    email: &str,
    password: &str,
) -> Result<(), cli::CliError> {
    let args = [
        "pocketratings",
        "user",
        "register",
        "--name",
        name,
        "--email",
        email,
        "--password",
        password,
    ]
    .into_iter()
    .map(std::ffi::OsString::from);

    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    cli::run(args, Some(pool), None, &mut stdout, &mut stderr).await
}

async fn run_set_password(
    pool: &sqlx::SqlitePool,
    email: &str,
    password: &str,
) -> (Result<(), cli::CliError>, String, String) {
    let args = [
        "pocketratings",
        "user",
        "set-password",
        "--email",
        email,
        "--password",
        password,
    ]
    .into_iter()
    .map(std::ffi::OsString::from);

    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let result = cli::run(args, Some(pool), None, &mut stdout, &mut stderr).await;
    let stdout_str = String::from_utf8(stdout.into_inner()).expect("stdout UTF-8");
    let stderr_str = String::from_utf8(stderr.into_inner()).expect("stderr UTF-8");
    (result, stdout_str, stderr_str)
}

#[tokio::test]
async fn set_password_updates_hash_and_verifies_new_password() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_set_password.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    run_register(&pool, "Alice", "alice@example.com", "oldpass")
        .await
        .expect("register");

    let (result, stdout, stderr) =
        run_set_password(&pool, "alice@example.com", "newpass").await;

    assert!(result.is_ok(), "expected Ok, stderr: {stderr}");
    assert!(stdout.contains("alice@example.com"));

    let user = db::user::get_by_email(&pool, "alice@example.com")
        .await
        .expect("get_by_email")
        .expect("user exists");
    assert!(user.verify_password("newpass").expect("verify new"));
    assert!(!user.verify_password("oldpass").expect("verify old"));
}

#[tokio::test]
async fn set_password_unknown_email_returns_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_set_password_unknown.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, _stdout, _stderr) =
        run_set_password(&pool, "nobody@example.com", "whatever").await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not found"),
        "error should mention not found: {err_msg}",
    );
}
