//! Integration tests for `pocketratings database backup` CLI.

use std::io::Cursor;

use pocketratings::cli;
use pocketratings::config::Config;
use pocketratings::db;

/// Run `pocketratings database backup` with optional extra args (e.g. `--output path`).
async fn run_db_backup(
    pool: &sqlx::SqlitePool,
    config_override: Option<&Config>,
    extra_args: &[&str],
) -> (Result<(), cli::CliError>, String, String) {
    let mut args: Vec<std::ffi::OsString> =
        vec!["pocketratings".into(), "database".into(), "backup".into()];
    for a in extra_args {
        args.push(std::ffi::OsString::from(*a));
    }

    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let result = cli::run(
        args.into_iter(),
        Some(pool),
        config_override,
        &mut stdout,
        &mut stderr,
    )
    .await;
    let stdout_str = String::from_utf8(stdout.into_inner()).expect("stdout UTF-8");
    let stderr_str = String::from_utf8(stderr.into_inner()).expect("stderr UTF-8");
    (result, stdout_str, stderr_str)
}

fn test_config(database_path: &str) -> Config {
    Config {
        database_path: database_path.to_string(),
        jwt_secret: "test-secret".to_string(),
        jwt_expiration_seconds: 3600,
        jwt_refresh_threshold_seconds: 600,
        bind: "127.0.0.1:3099".to_string(),
        pid_file: std::env::temp_dir()
            .join("pocketratings-db-backup-test.pid")
            .to_string_lossy()
            .into_owned(),
    }
}

/// Backup with default path creates `{DB_PATH}.backup` and prints path.
#[tokio::test]
async fn db_backup_creates_file_with_default_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("backup_test.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let backup_path = dir.path().join("backup_test.db.backup");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let config = test_config(db_path_str);
    let (result, stdout, stderr) = run_db_backup(&pool, Some(&config), &[]).await;

    assert!(result.is_ok(), "stderr: {stderr}");
    assert!(
        stdout.contains("backup written to"),
        "stdout should mention backup path: {stdout}",
    );
    let expected_suffix = format!("{db_path_str}.backup");
    assert!(
        stdout.trim().ends_with(&expected_suffix),
        "stdout should end with default backup path: {stdout}",
    );
    assert!(
        backup_path.exists(),
        "backup file should exist at {}",
        backup_path.display(),
    );

    // Backup should be valid SQLite (same schema as source).
    let backup_pool = db::create_pool(backup_path.to_str().expect("path"))
        .await
        .expect("open backup");
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'")
        .fetch_one(&backup_pool)
        .await
        .expect("query backup");
    assert!(row.0 >= 1, "backup should contain tables");
}

/// Backup with `--output` writes to the given path.
#[tokio::test]
async fn db_backup_creates_file_with_explicit_output() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("explicit.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let custom_backup = dir.path().join("custom_backup.db");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let custom_str = custom_backup.to_str().expect("path UTF-8");
    let config = test_config(db_path_str);
    let (result, stdout, stderr) =
        run_db_backup(&pool, Some(&config), &["--output", custom_str]).await;

    assert!(result.is_ok(), "stderr: {stderr}");
    assert!(stdout.contains("backup written to"), "stdout: {stdout}");
    assert!(
        stdout.contains(custom_str),
        "stdout should show path: {stdout}"
    );
    assert!(
        custom_backup.exists(),
        "backup file should exist at {}",
        custom_backup.display(),
    );
}

/// Backup with path containing `..` returns validation error.
#[tokio::test]
async fn db_backup_rejects_unsafe_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("unsafe_test.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let config = test_config(db_path_str);
    let (result, _stdout, _stderr) =
        run_db_backup(&pool, Some(&config), &["--output", "../evil.db"]).await;

    assert!(result.is_err(), "should reject path with ..");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("invalid") || err_msg.contains("refusing"),
        "error should mention invalid/refusing path: {err_msg}",
    );
}
