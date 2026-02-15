//! Integration tests for database setup and migrations.

use pocketratings::db;

/// Verify that migrations run successfully and the `users` table is created.
#[tokio::test]
async fn migrations_create_users_table() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let db_path_str = db_path
        .to_str()
        .expect("temp path is not valid UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("failed to create pool");

    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    // Query sqlite_master to check the users table exists.
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'users'",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to query sqlite_master");

    assert_eq!(row.0, 1, "users table should exist after migrations");
}
