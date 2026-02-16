//! Location persistence.
//!
//! Provides DB functions: [`get_by_id`], [`get_all`], [`get_all_with_deleted`],
//! [`insert`], [`update`], [`soft_delete`], and [`hard_delete`].

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::location::Location;

/// Map a DB row into a [`Location`]. Fails on invalid UUID or domain validation.
fn row_to_location(
    id: &str,
    name: &str,
    deleted_at: Option<i64>,
) -> Result<Location, crate::db::DbError> {
    let id = Uuid::parse_str(id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    Location::new(id, name.to_owned(), deleted_at)
        .map_err(|e| crate::db::DbError::InvalidData(e.to_string()))
}

/// Fetch a location by id (active only).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id(
    pool: &SqlitePool,
    id: Uuid,
) -> Result<Option<Location>, crate::db::DbError> {
    let id_str = id.to_string();
    let row = sqlx::query(
        "SELECT id, name, deleted_at FROM locations WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let id: String = row.get("id");
    let name: String = row.get("name");
    let deleted_at: Option<i64> = row.get("deleted_at");

    let location = row_to_location(&id, &name, deleted_at)?;
    Ok(Some(location))
}

/// Fetch all active locations.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Location>, crate::db::DbError> {
    let rows = sqlx::query("SELECT id, name, deleted_at FROM locations WHERE deleted_at IS NULL")
        .fetch_all(pool)
        .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let location = row_to_location(&id, &name, deleted_at)?;
        out.push(location);
    }
    Ok(out)
}

/// Fetch all locations (active and soft-deleted).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all_with_deleted(pool: &SqlitePool) -> Result<Vec<Location>, crate::db::DbError> {
    let rows = sqlx::query("SELECT id, name, deleted_at FROM locations")
        .fetch_all(pool)
        .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let location = row_to_location(&id, &name, deleted_at)?;
        out.push(location);
    }
    Ok(out)
}

/// Insert a location into the database.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure.
pub async fn insert(pool: &SqlitePool, location: &Location) -> Result<(), crate::db::DbError> {
    sqlx::query("INSERT INTO locations (id, name, deleted_at) VALUES (?, ?, ?)")
        .bind(location.id().to_string())
        .bind(location.name())
        .bind(location.deleted_at())
        .execute(pool)
        .await?;
    Ok(())
}

/// Update an existing location.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure.
pub async fn update(pool: &SqlitePool, location: &Location) -> Result<(), crate::db::DbError> {
    sqlx::query("UPDATE locations SET name = ?, deleted_at = ? WHERE id = ?")
        .bind(location.name())
        .bind(location.deleted_at())
        .bind(location.id().to_string())
        .execute(pool)
        .await?;
    Ok(())
}

/// Check that the location has no purchases. Returns error if it has any.
async fn ensure_no_purchases(
    pool: &SqlitePool,
    location_id: &str,
) -> Result<(), crate::db::DbError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM purchases WHERE location_id = ?")
        .bind(location_id)
        .fetch_one(pool)
        .await?;
    if count > 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "cannot delete location with purchases: {location_id}"
        )));
    }
    Ok(())
}

/// Soft-delete a location by id. Sets `deleted_at` to the current time.
///
/// Fails if the location has any purchases.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// the location has purchases or no active location exists with the given id.
pub async fn soft_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();
    ensure_no_purchases(pool, &id_str).await?;

    let now = chrono::Utc::now().timestamp();
    let result =
        sqlx::query("UPDATE locations SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(now)
            .bind(&id_str)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "location not found or already deleted: {id_str}"
        )));
    }

    Ok(())
}

/// Hard-delete a location by id (remove the row).
///
/// Fails if the location has any purchases.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// the location has purchases.
pub async fn hard_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();
    ensure_no_purchases(pool, &id_str).await?;

    let result = sqlx::query("DELETE FROM locations WHERE id = ?")
        .bind(&id_str)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "location not found: {id_str}"
        )));
    }

    Ok(())
}
