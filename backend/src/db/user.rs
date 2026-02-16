//! User persistence.
//!
//! Provides [`get_by_id`], [`get_by_email`], [`list_all`], [`insert`], [`soft_delete`], and [`hard_delete`] for loading, creating, and deleting users.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::user::User;

/// Map a DB row into a [`User`]. Fails on invalid UUID or domain validation.
fn row_to_user(
    id: &str,
    name: &str,
    email: &str,
    password: &str,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
) -> Result<User, crate::db::DbError> {
    let id = Uuid::parse_str(id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    User::new(
        id,
        name.to_owned(),
        email.to_owned(),
        password.to_owned(),
        created_at,
        updated_at,
        deleted_at,
    )
    .map_err(|e| crate::db::DbError::InvalidData(e.to_string()))
}

/// Fetch a user by id (active only).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<User>, crate::db::DbError> {
    let id_str = id.to_string();
    let row = sqlx::query(
        "SELECT id, name, email, password, created_at, updated_at, deleted_at FROM users WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let id: String = row.get("id");
    let name: String = row.get("name");
    let email: String = row.get("email");
    let password: String = row.get("password");
    let created_at: i64 = row.get("created_at");
    let updated_at: i64 = row.get("updated_at");
    let deleted_at: Option<i64> = row.get("deleted_at");

    let user = row_to_user(&id, &name, &email, &password, created_at, updated_at, deleted_at)?;
    Ok(Some(user))
}

/// Fetch a user by email (active only). Used for login.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_email(
    pool: &SqlitePool,
    email: &str,
) -> Result<Option<User>, crate::db::DbError> {
    let row = sqlx::query(
        "SELECT id, name, email, password, created_at, updated_at, deleted_at FROM users WHERE email = ? AND deleted_at IS NULL",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let id: String = row.get("id");
    let name: String = row.get("name");
    let email: String = row.get("email");
    let password: String = row.get("password");
    let created_at: i64 = row.get("created_at");
    let updated_at: i64 = row.get("updated_at");
    let deleted_at: Option<i64> = row.get("deleted_at");

    let user = row_to_user(&id, &name, &email, &password, created_at, updated_at, deleted_at)?;
    Ok(Some(user))
}

/// List users, ordered by email.
///
/// When `include_deleted` is `false` (default), only active users (`deleted_at` IS NULL) are returned.
/// When `true`, all users including soft-deleted are returned.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn list_all(
    pool: &SqlitePool,
    include_deleted: bool,
) -> Result<Vec<User>, crate::db::DbError> {
    let rows = if include_deleted {
        sqlx::query(
            "SELECT id, name, email, password, created_at, updated_at, deleted_at FROM users ORDER BY email",
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            "SELECT id, name, email, password, created_at, updated_at, deleted_at FROM users WHERE deleted_at IS NULL ORDER BY email",
        )
        .fetch_all(pool)
        .await?
    };

    let mut users = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let email: String = row.get("email");
        let password: String = row.get("password");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let user = row_to_user(&id, &name, &email, &password, created_at, updated_at, deleted_at)?;
        users.push(user);
    }
    Ok(users)
}

/// Insert a user into the database.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure (e.g. duplicate email).
pub async fn insert(pool: &SqlitePool, user: &User) -> Result<(), crate::db::DbError> {
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user.id().to_string())
    .bind(user.name())
    .bind(user.email())
    .bind(user.password())
    .bind(user.created_at())
    .bind(user.updated_at())
    .bind(user.deleted_at())
    .execute(pool)
    .await?;
    Ok(())
}

/// Soft-delete a user by id. Sets `deleted_at` and `updated_at` to the current time.
/// Only affects rows where `deleted_at` IS NULL.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if no active user exists with the given id.
pub async fn soft_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let now = chrono::Utc::now().timestamp();
    let id_str = id.to_string();
    let result = sqlx::query(
        "UPDATE users SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(now)
    .bind(&id_str)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "user not found or already deleted: {id_str}"
        )));
    }
    Ok(())
}

/// Permanently remove a user from the database (`DELETE`). Use with care.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if no user exists with the given id.
pub async fn hard_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&id_str)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "user not found: {id_str}"
        )));
    }
    Ok(())
}
