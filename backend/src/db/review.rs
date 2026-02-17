//! Review persistence.
//!
//! Provides DB functions: [`get_by_id`], [`list`], [`insert`], [`update`],
//! [`soft_delete`], and [`hard_delete`].

use rust_decimal::Decimal;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::review::Review;

/// Map a DB row into a [`Review`]. Fails on invalid UUID/Decimal or domain validation.
#[allow(clippy::too_many_arguments)]
fn row_to_review(
    id: &str,
    product_id: &str,
    user_id: &str,
    rating_str: &str,
    text: Option<&str>,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
) -> Result<Review, crate::db::DbError> {
    let id = Uuid::parse_str(id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let product_id =
        Uuid::parse_str(product_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let user_id =
        Uuid::parse_str(user_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let rating: Decimal = rating_str
        .parse()
        .map_err(|e: rust_decimal::Error| crate::db::DbError::InvalidData(e.to_string()))?;

    Review::new(
        id,
        product_id,
        user_id,
        rating,
        text.map(str::to_owned),
        created_at,
        updated_at,
        deleted_at,
    )
    .map_err(|e| crate::db::DbError::InvalidData(e.to_string()))
}

/// Fetch a review by id (active only).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Review>, crate::db::DbError> {
    let id_str = id.to_string();
    let row = sqlx::query(
        "SELECT id, product_id, user_id, rating, text, created_at, updated_at, deleted_at FROM reviews WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let id: String = row.get("id");
    let product_id: String = row.get("product_id");
    let user_id: String = row.get("user_id");
    let rating: String = row.get("rating");
    let text: Option<String> = row.get("text");
    let created_at: i64 = row.get("created_at");
    let updated_at: i64 = row.get("updated_at");
    let deleted_at: Option<i64> = row.get("deleted_at");

    let review = row_to_review(
        &id,
        &product_id,
        &user_id,
        &rating,
        text.as_deref(),
        created_at,
        updated_at,
        deleted_at,
    )?;
    Ok(Some(review))
}

/// List reviews with optional filters. Excludes soft-deleted unless `include_deleted`.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn list(
    pool: &SqlitePool,
    product_id: Option<Uuid>,
    user_id: Option<Uuid>,
    include_deleted: bool,
) -> Result<Vec<Review>, crate::db::DbError> {
    let rows = if include_deleted {
        match (product_id, user_id) {
            (None, None) => {
                sqlx::query("SELECT id, product_id, user_id, rating, text, created_at, updated_at, deleted_at FROM reviews ORDER BY updated_at DESC")
                    .fetch_all(pool)
                    .await?
            }
            (Some(pid), None) => {
                sqlx::query("SELECT id, product_id, user_id, rating, text, created_at, updated_at, deleted_at FROM reviews WHERE product_id = ? ORDER BY updated_at DESC")
                    .bind(pid.to_string())
                    .fetch_all(pool)
                    .await?
            }
            (None, Some(uid)) => {
                sqlx::query("SELECT id, product_id, user_id, rating, text, created_at, updated_at, deleted_at FROM reviews WHERE user_id = ? ORDER BY updated_at DESC")
                    .bind(uid.to_string())
                    .fetch_all(pool)
                    .await?
            }
            (Some(pid), Some(uid)) => {
                sqlx::query("SELECT id, product_id, user_id, rating, text, created_at, updated_at, deleted_at FROM reviews WHERE product_id = ? AND user_id = ? ORDER BY updated_at DESC")
                    .bind(pid.to_string())
                    .bind(uid.to_string())
                    .fetch_all(pool)
                    .await?
            }
        }
    } else {
        match (product_id, user_id) {
            (None, None) => {
                sqlx::query("SELECT id, product_id, user_id, rating, text, created_at, updated_at, deleted_at FROM reviews WHERE deleted_at IS NULL ORDER BY updated_at DESC")
                    .fetch_all(pool)
                    .await?
            }
            (Some(pid), None) => {
                sqlx::query("SELECT id, product_id, user_id, rating, text, created_at, updated_at, deleted_at FROM reviews WHERE product_id = ? AND deleted_at IS NULL ORDER BY updated_at DESC")
                    .bind(pid.to_string())
                    .fetch_all(pool)
                    .await?
            }
            (None, Some(uid)) => {
                sqlx::query("SELECT id, product_id, user_id, rating, text, created_at, updated_at, deleted_at FROM reviews WHERE user_id = ? AND deleted_at IS NULL ORDER BY updated_at DESC")
                    .bind(uid.to_string())
                    .fetch_all(pool)
                    .await?
            }
            (Some(pid), Some(uid)) => {
                sqlx::query("SELECT id, product_id, user_id, rating, text, created_at, updated_at, deleted_at FROM reviews WHERE product_id = ? AND user_id = ? AND deleted_at IS NULL ORDER BY updated_at DESC")
                    .bind(pid.to_string())
                    .bind(uid.to_string())
                    .fetch_all(pool)
                    .await?
            }
        }
    };

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let product_id: String = row.get("product_id");
        let user_id: String = row.get("user_id");
        let rating: String = row.get("rating");
        let text: Option<String> = row.get("text");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let review = row_to_review(
            &id,
            &product_id,
            &user_id,
            &rating,
            text.as_deref(),
            created_at,
            updated_at,
            deleted_at,
        )?;
        out.push(review);
    }
    Ok(out)
}

/// Insert a review into the database.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure.
pub async fn insert(pool: &SqlitePool, review: &Review) -> Result<(), crate::db::DbError> {
    sqlx::query(
        "INSERT INTO reviews (id, product_id, user_id, rating, text, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(review.id().to_string())
    .bind(review.product_id().to_string())
    .bind(review.user_id().to_string())
    .bind(review.rating().to_string())
    .bind(review.text())
    .bind(review.created_at())
    .bind(review.updated_at())
    .bind(review.deleted_at())
    .execute(pool)
    .await?;
    Ok(())
}

/// Update an existing review (rating, text, `updated_at`).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure.
pub async fn update(pool: &SqlitePool, review: &Review) -> Result<(), crate::db::DbError> {
    sqlx::query(
        "UPDATE reviews SET rating = ?, text = ?, updated_at = ?, deleted_at = ? WHERE id = ?",
    )
    .bind(review.rating().to_string())
    .bind(review.text())
    .bind(review.updated_at())
    .bind(review.deleted_at())
    .bind(review.id().to_string())
    .execute(pool)
    .await?;
    Ok(())
}

/// Soft-delete a review by id. Sets `deleted_at` to the current time.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// no active review exists with the given id.
pub async fn soft_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();
    let now = chrono::Utc::now().timestamp();
    let result =
        sqlx::query("UPDATE reviews SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(now)
            .bind(&id_str)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "review not found or already deleted: {id_str}"
        )));
    }

    Ok(())
}

/// Hard-delete a review by id (remove the row).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// no review exists with the given id.
pub async fn hard_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();
    let result = sqlx::query("DELETE FROM reviews WHERE id = ?")
        .bind(&id_str)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "review not found: {id_str}"
        )));
    }

    Ok(())
}
