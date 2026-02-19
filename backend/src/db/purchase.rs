//! Purchase persistence.
//!
//! Provides DB functions: [`get_by_id`], [`list`], [`insert`], [`soft_delete`], and [`hard_delete`].

use rust_decimal::Decimal;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::purchase::Purchase;

/// Map a DB row into a [`Purchase`]. Fails on invalid UUID/Decimal or domain validation.
#[allow(clippy::too_many_arguments)]
fn row_to_purchase(
    id: &str,
    user_id: &str,
    product_id: &str,
    location_id: &str,
    quantity: i32,
    price_str: &str,
    purchased_at: i64,
    deleted_at: Option<i64>,
) -> Result<Purchase, crate::db::DbError> {
    let id = Uuid::parse_str(id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let user_id =
        Uuid::parse_str(user_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let product_id =
        Uuid::parse_str(product_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let location_id =
        Uuid::parse_str(location_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let price: Decimal = price_str
        .parse()
        .map_err(|e: rust_decimal::Error| crate::db::DbError::InvalidData(e.to_string()))?;

    Purchase::new(
        id,
        user_id,
        product_id,
        location_id,
        quantity,
        price,
        purchased_at,
        deleted_at,
    )
    .map_err(|e| crate::db::DbError::InvalidData(e.to_string()))
}

/// Fetch a purchase by id (active only).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id(
    pool: &SqlitePool,
    id: Uuid,
) -> Result<Option<Purchase>, crate::db::DbError> {
    let id_str = id.to_string();
    let row = sqlx::query(
        "SELECT id, user_id, product_id, location_id, quantity, price, purchased_at, deleted_at FROM purchases WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let id: String = row.get("id");
    let user_id: String = row.get("user_id");
    let product_id: String = row.get("product_id");
    let location_id: String = row.get("location_id");
    let quantity: i32 = row.get("quantity");
    let price: String = row.get("price");
    let purchased_at: i64 = row.get("purchased_at");
    let deleted_at: Option<i64> = row.get("deleted_at");

    let purchase = row_to_purchase(
        &id,
        &user_id,
        &product_id,
        &location_id,
        quantity,
        &price,
        purchased_at,
        deleted_at,
    )?;
    Ok(Some(purchase))
}

/// List purchases with optional filters. Excludes soft-deleted unless `include_deleted`.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn list(
    pool: &SqlitePool,
    user_id: Option<Uuid>,
    product_id: Option<Uuid>,
    location_id: Option<Uuid>,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    include_deleted: bool,
) -> Result<Vec<Purchase>, crate::db::DbError> {
    let mut conditions = vec!["1=1".to_string()];
    let mut binds: Vec<String> = Vec::new();

    if let Some(uid) = user_id {
        conditions.push("user_id = ?".to_string());
        binds.push(uid.to_string());
    }
    if let Some(pid) = product_id {
        conditions.push("product_id = ?".to_string());
        binds.push(pid.to_string());
    }
    if let Some(lid) = location_id {
        conditions.push("location_id = ?".to_string());
        binds.push(lid.to_string());
    }
    if let Some(ts) = from_ts {
        conditions.push("purchased_at >= ?".to_string());
        binds.push(ts.to_string());
    }
    if let Some(ts) = to_ts {
        conditions.push("purchased_at <= ?".to_string());
        binds.push(ts.to_string());
    }
    if !include_deleted {
        conditions.push("deleted_at IS NULL".to_string());
    }

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "SELECT id, user_id, product_id, location_id, quantity, price, purchased_at, deleted_at FROM purchases WHERE {where_clause} ORDER BY purchased_at DESC"
    );

    let mut query = sqlx::query(&sql);
    for b in &binds {
        query = query.bind(b);
    }
    let rows = query.fetch_all(pool).await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let user_id: String = row.get("user_id");
        let product_id: String = row.get("product_id");
        let location_id: String = row.get("location_id");
        let quantity: i32 = row.get("quantity");
        let price: String = row.get("price");
        let purchased_at: i64 = row.get("purchased_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let purchase = row_to_purchase(
            &id,
            &user_id,
            &product_id,
            &location_id,
            quantity,
            &price,
            purchased_at,
            deleted_at,
        )?;
        out.push(purchase);
    }
    Ok(out)
}

/// Insert a purchase into the database.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure.
pub async fn insert(pool: &SqlitePool, purchase: &Purchase) -> Result<(), crate::db::DbError> {
    sqlx::query(
        "INSERT INTO purchases (id, user_id, product_id, location_id, quantity, price, purchased_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(purchase.id().to_string())
    .bind(purchase.user_id().to_string())
    .bind(purchase.product_id().to_string())
    .bind(purchase.location_id().to_string())
    .bind(purchase.quantity())
    .bind(purchase.price().to_string())
    .bind(purchase.purchased_at())
    .bind(purchase.deleted_at())
    .execute(pool)
    .await?;
    Ok(())
}

/// Update an existing purchase (all fields). Only updates active (non–soft-deleted) rows.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// no active purchase exists with the given id.
pub async fn update(pool: &SqlitePool, purchase: &Purchase) -> Result<(), crate::db::DbError> {
    let id_str = purchase.id().to_string();
    let result = sqlx::query(
        "UPDATE purchases SET user_id = ?, product_id = ?, location_id = ?, quantity = ?, price = ?, purchased_at = ? WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(purchase.user_id().to_string())
    .bind(purchase.product_id().to_string())
    .bind(purchase.location_id().to_string())
    .bind(purchase.quantity())
    .bind(purchase.price().to_string())
    .bind(purchase.purchased_at())
    .bind(&id_str)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "purchase not found or already deleted: {id_str}"
        )));
    }
    Ok(())
}

/// Soft-delete a purchase by id. Sets `deleted_at` to the current time.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// no active purchase exists with the given id.
pub async fn soft_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();
    let now = chrono::Utc::now().timestamp();
    let result =
        sqlx::query("UPDATE purchases SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(now)
            .bind(&id_str)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "purchase not found or already deleted: {id_str}"
        )));
    }

    Ok(())
}

/// Hard-delete a purchase by id (remove the row).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// no purchase exists with the given id.
pub async fn hard_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();
    let result = sqlx::query("DELETE FROM purchases WHERE id = ?")
        .bind(&id_str)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "purchase not found: {id_str}"
        )));
    }

    Ok(())
}
