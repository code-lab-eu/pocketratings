//! Integration tests for database setup and migrations.

use pocketratings::db;

/// Helper: count how many tables with the given name exist in sqlite_master.
async fn table_exists(pool: &sqlx::SqlitePool, table_name: &str) -> bool {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(table_name)
            .fetch_one(pool)
            .await
            .expect("failed to query sqlite_master");

    row.0 == 1
}

/// Verify that migrations run successfully and all expected tables are created.
#[tokio::test]
async fn migrations_create_all_tables() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let db_path_str = db_path.to_str().expect("temp path is not valid UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("failed to create pool");

    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let expected_tables = [
        "users",
        "categories",
        "products",
        "locations",
        "reviews",
        "purchases",
    ];

    for table in &expected_tables {
        assert!(
            table_exists(&pool, table).await,
            "table '{table}' should exist after migrations",
        );
    }
}

/// Verify that foreign keys are enforced (PRAGMA foreign_keys = ON).
#[tokio::test]
async fn foreign_keys_are_enabled() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("test_fk.db");
    let db_path_str = db_path.to_str().expect("temp path is not valid UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("failed to create pool");

    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let row: (i64,) = sqlx::query_as("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .expect("failed to query foreign_keys pragma");

    assert_eq!(row.0, 1, "foreign_keys should be enabled");
}
