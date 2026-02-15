//! User persistence.
//!
//! Provides [`get_by_id`] and [`get_by_email`] for loading users from the database.

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
