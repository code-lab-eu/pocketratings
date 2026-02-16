//! Integration tests for user DB functions.

use pocketratings::db;
use pocketratings::domain::user::User;
use uuid::Uuid;

/// Minimal PHC-style string so User::new accepts it (non-empty). Not used for verification.
const PLACEHOLDER_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$x$x";

#[tokio::test]
async fn get_by_id_returns_user_when_present() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("user_test.db");
    let db_path_str = db_path.to_str().expect("temp path is not valid UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("failed to create pool");
    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind("Alice")
    .bind("alice@example.com")
    .bind(PLACEHOLDER_HASH)
    .bind(1_000_i64)
    .bind(1_000_i64)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert user");

    let user = db::user::get_by_id(&pool, id)
        .await
        .expect("get_by_id")
        .expect("user should exist");
    assert_eq!(user.name(), "Alice");
    assert_eq!(user.email(), "alice@example.com");
    assert_eq!(user.id(), id);
    assert!(user.is_active());
}

#[tokio::test]
async fn get_by_email_returns_user_when_present() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("user_test2.db");
    let db_path_str = db_path.to_str().expect("temp path is not valid UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("failed to create pool");
    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind("Bob")
    .bind("bob@example.com")
    .bind(PLACEHOLDER_HASH)
    .bind(2_000_i64)
    .bind(2_000_i64)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert user");

    let user = db::user::get_by_email(&pool, "bob@example.com")
        .await
        .expect("get_by_email")
        .expect("user should exist");
    assert_eq!(user.name(), "Bob");
    assert_eq!(user.email(), "bob@example.com");
    assert_eq!(user.id(), id);
}

#[tokio::test]
async fn get_by_id_returns_none_for_unknown_id() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("user_test3.db");
    let db_path_str = db_path.to_str().expect("temp path is not valid UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("failed to create pool");
    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let result = db::user::get_by_id(&pool, Uuid::new_v4())
        .await
        .expect("get_by_id");
    assert!(result.is_none());
}

#[tokio::test]
async fn get_by_email_returns_none_for_unknown_email() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("user_test4.db");
    let db_path_str = db_path.to_str().expect("temp path is not valid UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("failed to create pool");
    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let result = db::user::get_by_email(&pool, "nobody@example.com")
        .await
        .expect("get_by_email");
    assert!(result.is_none());
}

#[tokio::test]
async fn soft_deleted_user_not_returned_by_get_by_id_or_get_by_email() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("user_test5.db");
    let db_path_str = db_path.to_str().expect("temp path is not valid UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("failed to create pool");
    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind("Deleted")
    .bind("deleted@example.com")
    .bind(PLACEHOLDER_HASH)
    .bind(1_000_i64)
    .bind(2_000_i64)
    .bind(3_000_i64)
    .execute(&pool)
    .await
    .expect("insert user");

    let by_id = db::user::get_by_id(&pool, id).await.expect("get_by_id");
    assert!(by_id.is_none(), "soft-deleted user should not be returned by get_by_id");

    let by_email = db::user::get_by_email(&pool, "deleted@example.com")
        .await
        .expect("get_by_email");
    assert!(
        by_email.is_none(),
        "soft-deleted user should not be returned by get_by_email"
    );
}

#[tokio::test]
async fn insert_persists_user_and_get_by_email_returns_it() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("user_insert_test.db");
    let db_path_str = db_path.to_str().expect("temp path is not valid UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("failed to create pool");
    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let id = Uuid::new_v4();
    let now = 5_000_i64;
    let user = User::new(
        id,
        "Carol".to_string(),
        "carol@example.com".to_string(),
        PLACEHOLDER_HASH.to_string(),
        now,
        now,
        None,
    )
    .expect("valid user");

    db::user::insert(&pool, &user).await.expect("insert");

    let loaded = db::user::get_by_email(&pool, "carol@example.com")
        .await
        .expect("get_by_email")
        .expect("user should exist");
    assert_eq!(loaded.id(), id);
    assert_eq!(loaded.name(), "Carol");
    assert_eq!(loaded.email(), "carol@example.com");
}

#[tokio::test]
async fn list_all_returns_only_active_users_by_default() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("user_list_active.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id_active = Uuid::new_v4();
    let id_deleted = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id_active.to_string())
    .bind("Active")
    .bind("active@example.com")
    .bind(PLACEHOLDER_HASH)
    .bind(1_000_i64)
    .bind(1_000_i64)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert active");
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id_deleted.to_string())
    .bind("Deleted")
    .bind("deleted@example.com")
    .bind(PLACEHOLDER_HASH)
    .bind(2_000_i64)
    .bind(2_000_i64)
    .bind(3_000_i64)
    .execute(&pool)
    .await
    .expect("insert deleted");

    let users = db::user::list_all(&pool, false).await.expect("list_all");
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].email(), "active@example.com");
}

#[tokio::test]
async fn list_all_with_include_deleted_returns_soft_deleted_users() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("user_list_include_deleted.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id_deleted = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id_deleted.to_string())
    .bind("Deleted")
    .bind("deleted@example.com")
    .bind(PLACEHOLDER_HASH)
    .bind(1_000_i64)
    .bind(2_000_i64)
    .bind(3_000_i64)
    .execute(&pool)
    .await
    .expect("insert deleted");

    let users = db::user::list_all(&pool, true).await.expect("list_all");
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].email(), "deleted@example.com");
    assert!(!users[0].is_active());
}

#[tokio::test]
async fn list_all_ordered_by_email() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("user_list_order.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    for (name, email) in [("Alice", "alice@example.com"), ("Bob", "bob@example.com")] {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(name)
        .bind(email)
        .bind(PLACEHOLDER_HASH)
        .bind(1_000_i64)
        .bind(1_000_i64)
        .bind::<Option<i64>>(None)
        .execute(&pool)
        .await
        .expect("insert");
    }

    let users = db::user::list_all(&pool, false).await.expect("list_all");
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].email(), "alice@example.com");
    assert_eq!(users[1].email(), "bob@example.com");
}

#[tokio::test]
async fn soft_delete_sets_deleted_at_and_user_not_returned_by_get_by_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("user_soft_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind("ToDelete")
    .bind("delete@example.com")
    .bind(PLACEHOLDER_HASH)
    .bind(1_000_i64)
    .bind(1_000_i64)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert");

    db::user::soft_delete(&pool, id).await.expect("soft_delete");

    let got = db::user::get_by_id(&pool, id).await.expect("get_by_id");
    assert!(got.is_none(), "soft-deleted user should not be returned by get_by_id");

    let with_deleted = db::user::list_all(&pool, true).await.expect("list_all");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
async fn soft_delete_returns_error_for_unknown_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("user_soft_delete_unknown.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let result = db::user::soft_delete(&pool, Uuid::new_v4()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn soft_delete_returns_error_for_already_deleted_user() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("user_soft_delete_twice.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind("ToDelete")
    .bind("twice@example.com")
    .bind(PLACEHOLDER_HASH)
    .bind(1_000_i64)
    .bind(1_000_i64)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert");

    db::user::soft_delete(&pool, id).await.expect("first soft_delete");
    let second = db::user::soft_delete(&pool, id).await;
    assert!(second.is_err(), "second soft_delete should fail (already deleted)");
}

#[tokio::test]
async fn hard_delete_removes_row_from_database() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("user_hard_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind("ToRemove")
    .bind("remove@example.com")
    .bind(PLACEHOLDER_HASH)
    .bind(1_000_i64)
    .bind(1_000_i64)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert");

    db::user::hard_delete(&pool, id).await.expect("hard_delete");

    let got = db::user::get_by_id(&pool, id).await.expect("get_by_id");
    assert!(got.is_none());
    let all = db::user::list_all(&pool, true).await.expect("list_all");
    assert!(!all.iter().any(|u| u.id() == id));
}

#[tokio::test]
async fn hard_delete_returns_error_for_unknown_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("user_hard_delete_unknown.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let result = db::user::hard_delete(&pool, Uuid::new_v4()).await;
    assert!(result.is_err());
}
