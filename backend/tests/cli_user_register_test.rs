//! Integration tests for `pocketratings user register` CLI.

use std::io::Cursor;

use pocketratings::cli;
use pocketratings::db;

async fn run_register(
    pool: &sqlx::SqlitePool,
    name: &str,
    email: &str,
    password: &str,
    output_json: bool,
) -> (Result<(), cli::CliError>, String, String) {
    let mut args: Vec<std::ffi::OsString> = [
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
    .map(std::ffi::OsString::from)
    .collect();
    if output_json {
        args.push(std::ffi::OsString::from("--output"));
        args.push(std::ffi::OsString::from("json"));
    }

    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let result = cli::run(args.into_iter(), Some(pool), &mut stdout, &mut stderr).await;
    let stdout_str = String::from_utf8(stdout.into_inner()).expect("stdout UTF-8");
    let stderr_str = String::from_utf8(stderr.into_inner()).expect("stderr UTF-8");
    (result, stdout_str, stderr_str)
}

#[tokio::test]
async fn register_success_exit_0_and_stdout_contains_email() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_register.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, stdout, stderr) =
        run_register(&pool, "Alice", "alice@example.com", "secret", false).await;

    assert!(result.is_ok(), "expected Ok, stderr: {}", stderr);
    assert!(stdout.contains("registered") || stdout.contains("alice@example.com"));
    assert!(stderr.is_empty());
}

#[tokio::test]
async fn register_then_get_by_email_and_verify_password() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_register_verify.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, _, _) = run_register(&pool, "Bob", "bob@example.com", "mypassword", false).await;
    result.expect("first register");

    let user = db::user::get_by_email(&pool, "bob@example.com")
        .await
        .expect("get_by_email")
        .expect("user exists");
    assert_eq!(user.name(), "Bob");
    assert!(user.verify_password("mypassword").expect("verify"));
}

#[tokio::test]
async fn duplicate_email_returns_error_and_stderr_contains_already_registered() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_dup.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (r1, _, _) = run_register(&pool, "First", "same@example.com", "pass", false).await;
    r1.expect("first register");

    let (r2, _stdout, _stderr) =
        run_register(&pool, "Second", "same@example.com", "other", false).await;

    assert!(r2.is_err());
    let err_msg = r2.unwrap_err().to_string();
    assert!(
        err_msg.contains("already registered"),
        "error should mention already registered: {}",
        err_msg
    );
}

#[tokio::test]
async fn invalid_email_returns_error_and_stderr_contains_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_invalid_email.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, _stdout, _stderr) = run_register(&pool, "X", "notanemail", "pass", false).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(!err_msg.is_empty(), "error should contain message");
}

#[tokio::test]
async fn output_json_produces_valid_json_with_id_and_email() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_json.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, stdout, stderr) =
        run_register(&pool, "Carol", "carol@example.com", "secret", true).await;

    assert!(result.is_ok(), "stderr: {}", stderr);
    assert!(stderr.is_empty());
    let line = stdout.lines().next().expect("at least one line");
    let json: serde_json::Value = serde_json::from_str(line).expect("valid JSON");
    assert!(json.get("id").and_then(|v| v.as_str()).is_some());
    assert_eq!(
        json.get("email").and_then(|v| v.as_str()),
        Some("carol@example.com")
    );
}
