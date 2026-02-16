//! Integration tests for `pocketratings user list` CLI.

use std::io::Cursor;

use pocketratings::cli;
use pocketratings::db;

const PLACEHOLDER_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$x$x";

async fn run_list(
    pool: &sqlx::SqlitePool,
    output_json: bool,
    include_deleted: bool,
) -> (Result<(), cli::CliError>, String, String) {
    let mut args: Vec<std::ffi::OsString> = ["pocketratings", "user", "list"]
        .into_iter()
        .map(std::ffi::OsString::from)
        .collect();
    if output_json {
        args.push(std::ffi::OsString::from("--output"));
        args.push(std::ffi::OsString::from("json"));
    }
    if include_deleted {
        args.push(std::ffi::OsString::from("--include-deleted"));
    }

    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let result = cli::run(args.into_iter(), Some(pool), &mut stdout, &mut stderr).await;
    let stdout_str = String::from_utf8(stdout.into_inner()).expect("stdout UTF-8");
    let stderr_str = String::from_utf8(stderr.into_inner()).expect("stderr UTF-8");
    (result, stdout_str, stderr_str)
}

#[tokio::test]
async fn list_success_empty_when_no_users() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_list_empty.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, stdout, stderr) = run_list(&pool, false, false).await;

    assert!(result.is_ok(), "stderr: {}", stderr);
    assert!(stderr.is_empty());
    assert!(stdout.trim().is_empty(), "stdout should be empty: {:?}", stdout);
}

#[tokio::test]
async fn list_shows_registered_users() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_list_show.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    cli::run(
        [
            "pocketratings",
            "user",
            "register",
            "--name",
            "Alice",
            "--email",
            "alice@example.com",
            "--password",
            "secret",
        ]
        .into_iter()
        .map(std::ffi::OsString::from),
        Some(&pool),
        &mut Cursor::new(Vec::new()),
        &mut Cursor::new(Vec::new()),
    )
    .await
    .expect("register");

    let (result, stdout, stderr) = run_list(&pool, false, false).await;

    assert!(result.is_ok(), "stderr: {}", stderr);
    assert!(stdout.contains("alice@example.com"));
    assert!(stdout.contains("Alice"));
}

#[tokio::test]
async fn list_output_json_produces_valid_json_array() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_list_json.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    cli::run(
        [
            "pocketratings",
            "user",
            "register",
            "--name",
            "Bob",
            "--email",
            "bob@example.com",
            "--password",
            "secret",
        ]
        .into_iter()
        .map(std::ffi::OsString::from),
        Some(&pool),
        &mut Cursor::new(Vec::new()),
        &mut Cursor::new(Vec::new()),
    )
    .await
    .expect("register");

    let (result, stdout, stderr) = run_list(&pool, true, false).await;

    assert!(result.is_ok(), "stderr: {}", stderr);
    let line = stdout.lines().next().expect("at least one line");
    let arr: Vec<serde_json::Value> = serde_json::from_str(line).expect("valid JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].get("email").and_then(|v| v.as_str()), Some("bob@example.com"));
    assert_eq!(arr[0].get("name").and_then(|v| v.as_str()), Some("Bob"));
    assert!(arr[0].get("id").and_then(|v| v.as_str()).is_some());
}

#[tokio::test]
async fn list_include_deleted_includes_soft_deleted_user() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_list_include_deleted.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let deleted_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(deleted_id.to_string())
    .bind("Deleted User")
    .bind("deleted@example.com")
    .bind(PLACEHOLDER_HASH)
    .bind(1_000_i64)
    .bind(2_000_i64)
    .bind(3_000_i64)
    .execute(&pool)
    .await
    .expect("insert deleted user");

    let (result_default, stdout_default, _) = run_list(&pool, false, false).await;
    assert!(result_default.is_ok());
    assert!(
        !stdout_default.contains("deleted@example.com"),
        "default list should not include deleted: {}",
        stdout_default
    );

    let (result_include, stdout_include, stderr) = run_list(&pool, false, true).await;
    assert!(result_include.is_ok(), "stderr: {}", stderr);
    assert!(
        stdout_include.contains("deleted@example.com"),
        "list --include-deleted should include deleted user: {}",
        stdout_include
    );
}
