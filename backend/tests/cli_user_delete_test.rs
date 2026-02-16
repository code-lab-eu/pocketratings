//! Integration tests for `pocketratings user delete` CLI.

use std::io::Cursor;

use pocketratings::cli;
use pocketratings::db;

async fn run_delete(
    pool: &sqlx::SqlitePool,
    id: &str,
    force: bool,
) -> (Result<(), cli::CliError>, String, String) {
    let mut args: Vec<std::ffi::OsString> = ["pocketratings", "user", "delete", id]
        .into_iter()
        .map(std::ffi::OsString::from)
        .collect();
    if force {
        args.push(std::ffi::OsString::from("--force"));
    }

    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let result = cli::run(args.into_iter(), Some(pool), &mut stdout, &mut stderr).await;
    let stdout_str = String::from_utf8(stdout.into_inner()).expect("stdout UTF-8");
    let stderr_str = String::from_utf8(stderr.into_inner()).expect("stderr UTF-8");
    (result, stdout_str, stderr_str)
}

#[tokio::test]
async fn delete_success_soft_deletes_user() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_delete_ok.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let mut register_stdout = Cursor::new(Vec::new());
    cli::run(
        [
            "pocketratings",
            "user",
            "register",
            "--name",
            "ToDelete",
            "--email",
            "delete@example.com",
            "--password",
            "secret",
            "--output",
            "json",
        ]
        .into_iter()
        .map(std::ffi::OsString::from),
        Some(&pool),
        &mut register_stdout,
        &mut Cursor::new(Vec::new()),
    )
    .await
    .expect("register");
    let out = String::from_utf8(register_stdout.into_inner()).expect("UTF-8");
    let json: serde_json::Value = serde_json::from_str(out.trim()).expect("JSON");
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id in response");

    let (result, stdout, stderr) = run_delete(&pool, id, false).await;

    assert!(result.is_ok(), "stderr: {}", stderr);
    assert!(stdout.contains("deleted") || stdout.contains(id));
    let got = db::user::get_by_id(&pool, id.parse().expect("uuid"))
        .await
        .expect("get_by_id");
    assert!(got.is_none(), "user should be soft-deleted");
}

#[tokio::test]
async fn delete_invalid_uuid_returns_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_delete_invalid.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, _stdout, _stderr) = run_delete(&pool, "not-a-uuid", false).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.to_lowercase().contains("invalid") || err_msg.contains("id"));
}

#[tokio::test]
async fn delete_unknown_id_returns_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_delete_unknown.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let unknown_id = uuid::Uuid::new_v4().to_string();
    let (result, _stdout, _stderr) = run_delete(&pool, &unknown_id, false).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not found") || err_msg.contains("already deleted"),
        "error should mention not found or already deleted: {}",
        err_msg
    );
}

#[tokio::test]
async fn delete_force_removes_user_from_database() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_delete_force.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let mut register_stdout = Cursor::new(Vec::new());
    cli::run(
        [
            "pocketratings",
            "user",
            "register",
            "--name",
            "ToRemove",
            "--email",
            "remove@example.com",
            "--password",
            "secret",
            "--output",
            "json",
        ]
        .into_iter()
        .map(std::ffi::OsString::from),
        Some(&pool),
        &mut register_stdout,
        &mut Cursor::new(Vec::new()),
    )
    .await
    .expect("register");
    let out = String::from_utf8(register_stdout.into_inner()).expect("UTF-8");
    let json: serde_json::Value = serde_json::from_str(out.trim()).expect("JSON");
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id in response");

    let (result, stdout, stderr) = run_delete(&pool, id, true).await;

    assert!(result.is_ok(), "stderr: {}", stderr);
    assert!(stdout.contains("removed") || stdout.contains(id));
    let got = db::user::get_by_id(&pool, id.parse().expect("uuid"))
        .await
        .expect("get_by_id");
    assert!(got.is_none(), "user should be removed");
    let with_deleted = db::user::list_all(&pool, true).await.expect("list_all");
    assert!(
        !with_deleted
            .iter()
            .any(|u| u.email() == "remove@example.com"),
        "user should not appear even in list with include_deleted"
    );
}
