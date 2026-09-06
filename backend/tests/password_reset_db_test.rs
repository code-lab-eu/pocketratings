//! Integration tests for password reset token DB functions.

use pocketratings::auth::password;
use pocketratings::auth::reset_token::TOKEN_TTL_SECONDS;
use pocketratings::db;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

const NOW: i64 = 1_700_000_000;

/// Create a migrated temp-file pool. Keep the returned `TempDir` alive for the whole test.
async fn test_pool(name: &str) -> (SqlitePool, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join(format!("{name}.db"));
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");
    (pool, dir)
}

/// Insert an active user with the given password and return its id.
async fn insert_user(pool: &SqlitePool, email: &str, plain_password: &str) -> Uuid {
    let id = Uuid::new_v4();
    let hash = password::hash_password(plain_password).expect("hash");
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind("Alice")
    .bind(email)
    .bind(&hash)
    .bind(NOW)
    .bind(NOW)
    .bind::<Option<i64>>(None)
    .execute(pool)
    .await
    .expect("insert user");
    id
}

/// Read the stored password hash of a user.
async fn stored_password(pool: &SqlitePool, user_id: Uuid) -> String {
    sqlx::query("SELECT password FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await
        .expect("select password")
        .get("password")
}

#[tokio::test]
async fn create_stores_token_with_ttl_expiry_and_is_valid() {
    let (pool, _dir) = test_pool("reset_create").await;
    let user_id = insert_user(&pool, "alice@example.com", "oldpass").await;

    db::password_reset::create(&pool, user_id, "hash-1", NOW)
        .await
        .expect("create");

    let row = sqlx::query("SELECT user_id, expires_at, used_at, created_at FROM password_reset_tokens WHERE token_hash = ?")
        .bind("hash-1")
        .fetch_one(&pool)
        .await
        .expect("select token");
    let stored_user_id: String = row.get("user_id");
    let expires_at: i64 = row.get("expires_at");
    let used_at: Option<i64> = row.get("used_at");
    let created_at: i64 = row.get("created_at");
    assert_eq!(stored_user_id, user_id.to_string());
    assert_eq!(expires_at, NOW + TOKEN_TTL_SECONDS);
    assert_eq!(used_at, None);
    assert_eq!(created_at, NOW);

    assert!(
        db::password_reset::is_valid(&pool, "hash-1", NOW)
            .await
            .expect("is_valid")
    );
}

#[tokio::test]
async fn is_valid_is_false_for_unknown_and_expired_tokens() {
    let (pool, _dir) = test_pool("reset_is_valid").await;
    let user_id = insert_user(&pool, "alice@example.com", "oldpass").await;
    db::password_reset::create(&pool, user_id, "hash-1", NOW)
        .await
        .expect("create");

    assert!(
        !db::password_reset::is_valid(&pool, "unknown-hash", NOW)
            .await
            .expect("is_valid unknown")
    );
    assert!(
        !db::password_reset::is_valid(&pool, "hash-1", NOW + TOKEN_TTL_SECONDS + 1)
            .await
            .expect("is_valid expired")
    );
}

#[tokio::test]
async fn is_valid_is_false_for_a_soft_deleted_user() {
    let (pool, _dir) = test_pool("reset_is_valid_deleted").await;
    let user_id = insert_user(&pool, "alice@example.com", "oldpass").await;
    db::password_reset::create(&pool, user_id, "hash-1", NOW)
        .await
        .expect("create");
    db::user::soft_delete(&pool, user_id)
        .await
        .expect("soft delete");

    assert!(
        !db::password_reset::is_valid(&pool, "hash-1", NOW)
            .await
            .expect("is_valid")
    );
}

#[tokio::test]
async fn consume_sets_the_password_marks_the_token_used_and_returns_true() {
    let (pool, _dir) = test_pool("reset_consume").await;
    let user_id = insert_user(&pool, "alice@example.com", "oldpass").await;
    db::password_reset::create(&pool, user_id, "hash-1", NOW)
        .await
        .expect("create");
    let new_hash = password::hash_password("newpass").expect("hash");

    let consumed = db::password_reset::consume_and_set_password(&pool, "hash-1", &new_hash, NOW)
        .await
        .expect("consume");

    assert!(consumed);
    assert_eq!(stored_password(&pool, user_id).await, new_hash);
    let used_at: Option<i64> =
        sqlx::query("SELECT used_at FROM password_reset_tokens WHERE token_hash = ?")
            .bind("hash-1")
            .fetch_one(&pool)
            .await
            .expect("select used_at")
            .get("used_at");
    assert_eq!(used_at, Some(NOW));
    assert!(
        !db::password_reset::is_valid(&pool, "hash-1", NOW)
            .await
            .expect("is_valid after consume")
    );
}

#[tokio::test]
async fn second_consume_returns_false_and_leaves_the_password_unchanged() {
    let (pool, _dir) = test_pool("reset_consume_twice").await;
    let user_id = insert_user(&pool, "alice@example.com", "oldpass").await;
    db::password_reset::create(&pool, user_id, "hash-1", NOW)
        .await
        .expect("create");
    let first_hash = password::hash_password("newpass").expect("hash");
    db::password_reset::consume_and_set_password(&pool, "hash-1", &first_hash, NOW)
        .await
        .expect("first consume");

    let second_hash = password::hash_password("evenneweRpass").expect("hash");
    let consumed =
        db::password_reset::consume_and_set_password(&pool, "hash-1", &second_hash, NOW + 1)
            .await
            .expect("second consume");

    assert!(!consumed);
    assert_eq!(stored_password(&pool, user_id).await, first_hash);
}

#[tokio::test]
async fn consume_of_an_expired_token_returns_false_and_leaves_the_password_unchanged() {
    let (pool, _dir) = test_pool("reset_consume_expired").await;
    let user_id = insert_user(&pool, "alice@example.com", "oldpass").await;
    let original = stored_password(&pool, user_id).await;
    db::password_reset::create(&pool, user_id, "hash-1", NOW)
        .await
        .expect("create");
    let new_hash = password::hash_password("newpass").expect("hash");

    let consumed = db::password_reset::consume_and_set_password(
        &pool,
        "hash-1",
        &new_hash,
        NOW + TOKEN_TTL_SECONDS + 1,
    )
    .await
    .expect("consume");

    assert!(!consumed);
    assert_eq!(stored_password(&pool, user_id).await, original);
}

#[tokio::test]
async fn consume_of_an_unknown_token_returns_false() {
    let (pool, _dir) = test_pool("reset_consume_unknown").await;
    let user_id = insert_user(&pool, "alice@example.com", "oldpass").await;
    let original = stored_password(&pool, user_id).await;
    let new_hash = password::hash_password("newpass").expect("hash");

    let consumed =
        db::password_reset::consume_and_set_password(&pool, "unknown-hash", &new_hash, NOW)
            .await
            .expect("consume");

    assert!(!consumed);
    assert_eq!(stored_password(&pool, user_id).await, original);
}

#[tokio::test]
async fn creating_a_new_token_leaves_an_earlier_one_valid() {
    let (pool, _dir) = test_pool("reset_two_tokens").await;
    let user_id = insert_user(&pool, "alice@example.com", "oldpass").await;
    db::password_reset::create(&pool, user_id, "hash-1", NOW)
        .await
        .expect("create first");

    db::password_reset::create(&pool, user_id, "hash-2", NOW + 60)
        .await
        .expect("create second");

    assert!(
        db::password_reset::is_valid(&pool, "hash-1", NOW + 60)
            .await
            .expect("first still valid")
    );
    assert!(
        db::password_reset::is_valid(&pool, "hash-2", NOW + 60)
            .await
            .expect("second valid")
    );
}
