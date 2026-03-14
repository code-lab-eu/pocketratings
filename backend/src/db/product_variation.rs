//! Product variation persistence.
//!
//! Provides DB functions: [`get_by_id`], [`list_by_product_id`], [`insert`],
//! [`update`], [`soft_delete`], [`count_by_product_id`], and [`ensure_no_purchases`].

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::product_variation::ProductVariation;

/// Map a DB row into a [`ProductVariation`]. Fails on invalid UUID or domain validation.
#[allow(clippy::too_many_arguments)]
fn row_to_variation(
    id: &str,
    product_id: &str,
    label: &str,
    unit: &str,
    quantity: Option<u32>,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
) -> Result<ProductVariation, crate::db::DbError> {
    let id = Uuid::parse_str(id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let product_id =
        Uuid::parse_str(product_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    ProductVariation::new(
        id, product_id, label, unit, quantity, created_at, updated_at, deleted_at,
    )
    .map_err(|e| crate::db::DbError::InvalidData(e.to_string()))
}

/// Fetch a variation by id.
///
/// When `include_deleted` is `false`, only active variations are returned.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id(
    pool: &SqlitePool,
    id: Uuid,
    include_deleted: bool,
) -> Result<Option<ProductVariation>, crate::db::DbError> {
    let id_str = id.to_string();
    let row = if include_deleted {
        sqlx::query(
            "SELECT id, product_id, label, unit, quantity, created_at, updated_at, deleted_at \
             FROM product_variations WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query(
            "SELECT id, product_id, label, unit, quantity, created_at, updated_at, deleted_at \
             FROM product_variations WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(&id_str)
        .fetch_optional(pool)
        .await?
    };

    let Some(row) = row else {
        return Ok(None);
    };

    let id: String = row.get("id");
    let product_id: String = row.get("product_id");
    let label: String = row.get("label");
    let unit: String = row.get("unit");
    // SQLite INTEGER is i64; domain uses Option<u32>, so convert (negative -> None).
    let quantity: Option<i64> = row.get("quantity");
    let quantity = quantity.and_then(|q| u32::try_from(q).ok());
    let created_at: i64 = row.get("created_at");
    let updated_at: i64 = row.get("updated_at");
    let deleted_at: Option<i64> = row.get("deleted_at");

    let variation = row_to_variation(
        &id,
        &product_id,
        &label,
        &unit,
        quantity,
        created_at,
        updated_at,
        deleted_at,
    )?;
    Ok(Some(variation))
}

/// List variations for a product, ordered by `created_at`.
///
/// When `include_deleted` is `false`, only active variations are returned.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn list_by_product_id(
    pool: &SqlitePool,
    product_id: Uuid,
    include_deleted: bool,
) -> Result<Vec<ProductVariation>, crate::db::DbError> {
    let product_id_str = product_id.to_string();
    let rows = if include_deleted {
        sqlx::query(
            "SELECT id, product_id, label, unit, quantity, created_at, updated_at, deleted_at \
             FROM product_variations WHERE product_id = ? ORDER BY created_at",
        )
        .bind(&product_id_str)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            "SELECT id, product_id, label, unit, quantity, created_at, updated_at, deleted_at \
             FROM product_variations WHERE product_id = ? AND deleted_at IS NULL ORDER BY created_at",
        )
        .bind(&product_id_str)
        .fetch_all(pool)
        .await?
    };

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let product_id: String = row.get("product_id");
        let label: String = row.get("label");
        let unit: String = row.get("unit");
        // SQLite INTEGER is i64; domain uses Option<u32>, so convert (negative -> None).
        let quantity: Option<i64> = row.get("quantity");
        let quantity = quantity.and_then(|q| u32::try_from(q).ok());
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let variation = row_to_variation(
            &id,
            &product_id,
            &label,
            &unit,
            quantity,
            created_at,
            updated_at,
            deleted_at,
        )?;
        out.push(variation);
    }
    Ok(out)
}

/// Insert a product variation.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or domain validation failure.
pub async fn insert(
    pool: &SqlitePool,
    variation: &ProductVariation,
) -> Result<(), crate::db::DbError> {
    sqlx::query(
        "INSERT INTO product_variations (id, product_id, label, unit, quantity, created_at, updated_at, deleted_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(variation.id().to_string())
    .bind(variation.product_id().to_string())
    .bind(variation.label())
    .bind(variation.unit())
    .bind(variation.quantity().map(i64::from))
    .bind(variation.created_at())
    .bind(variation.updated_at())
    .bind(variation.deleted_at())
    .execute(pool)
    .await?;
    Ok(())
}

/// Update an existing variation. Only updates active (non–soft-deleted) rows.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure or if no active row exists.
pub async fn update(
    pool: &SqlitePool,
    variation: &ProductVariation,
) -> Result<(), crate::db::DbError> {
    let id_str = variation.id().to_string();
    let result = sqlx::query(
        "UPDATE product_variations \
         SET product_id = ?, label = ?, unit = ?, quantity = ?, updated_at = ?, deleted_at = ? \
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(variation.product_id().to_string())
    .bind(variation.label())
    .bind(variation.unit())
    .bind(variation.quantity().map(i64::from))
    .bind(variation.updated_at())
    .bind(variation.deleted_at())
    .bind(&id_str)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "product variation not found or already deleted: {id_str}"
        )));
    }
    Ok(())
}

/// Soft-delete a variation by id.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] if no active row exists.
pub async fn soft_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();
    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE product_variations SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(&id_str)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "product variation not found or already deleted: {id_str}"
        )));
    }
    Ok(())
}

/// Count variations for a product (optionally including soft-deleted).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure.
pub async fn count_by_product_id(
    pool: &SqlitePool,
    product_id: Uuid,
    include_deleted: bool,
) -> Result<i64, crate::db::DbError> {
    let product_id_str = product_id.to_string();
    let count: (i64,) = if include_deleted {
        sqlx::query_as("SELECT COUNT(*) FROM product_variations WHERE product_id = ?")
            .bind(&product_id_str)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_as(
            "SELECT COUNT(*) FROM product_variations WHERE product_id = ? AND deleted_at IS NULL",
        )
        .bind(&product_id_str)
        .fetch_one(pool)
        .await?
    };
    Ok(count.0)
}

/// Ensure the variation has no purchases. Returns error if any purchase references it.
///
/// # Errors
///
/// Returns [`crate::db::DbError::InvalidData`] with message if the variation has purchases.
pub async fn ensure_no_purchases(
    pool: &SqlitePool,
    variation_id: Uuid,
) -> Result<(), crate::db::DbError> {
    let variation_id_str = variation_id.to_string();
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM purchases WHERE variation_id = ? AND deleted_at IS NULL",
    )
    .bind(&variation_id_str)
    .fetch_one(pool)
    .await?;

    if count.0 > 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "cannot delete variation with purchases: {variation_id_str}"
        )));
    }
    Ok(())
}
