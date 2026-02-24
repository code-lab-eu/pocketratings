//! Purchase persistence.
//!
//! Provides DB functions: [`get_by_id`], [`get_by_id_with_relations`], [`list`],
//! [`list_with_relations`], [`insert`], [`soft_delete`], and [`hard_delete`].

use rust_decimal::Decimal;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::purchase::Purchase;

/// One purchase row with joined user, product, and location names for API responses.
#[derive(Debug, Clone)]
pub struct PurchaseWithRelations {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub quantity: i32,
    pub price: String,
    pub purchased_at: i64,
    pub deleted_at: Option<i64>,
    pub user_name: String,
    pub product_brand: String,
    pub product_name: String,
    pub location_name: String,
}

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

const PURCHASE_JOIN_SELECT: &str = "SELECT p.id, p.user_id, p.product_id, p.location_id, p.quantity, p.price, p.purchased_at, p.deleted_at, \
    u.name AS user_name, prod.brand AS product_brand, prod.name AS product_name, loc.name AS location_name ";
const PURCHASE_JOIN_FROM: &str = "FROM purchases p \
    JOIN users u ON p.user_id = u.id \
    JOIN products prod ON p.product_id = prod.id \
    JOIN locations loc ON p.location_id = loc.id";

fn row_to_purchase_with_relations(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<PurchaseWithRelations, crate::db::DbError> {
    let id: String = row.get("id");
    let user_id: String = row.get("user_id");
    let product_id: String = row.get("product_id");
    let location_id: String = row.get("location_id");
    let quantity: i32 = row.get("quantity");
    let price: String = row.get("price");
    let purchased_at: i64 = row.get("purchased_at");
    let deleted_at: Option<i64> = row.get("deleted_at");
    let user_name: String = row.get("user_name");
    let product_brand: String = row.get("product_brand");
    let product_name: String = row.get("product_name");
    let location_name: String = row.get("location_name");

    let id = Uuid::parse_str(&id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let user_id =
        Uuid::parse_str(&user_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let product_id =
        Uuid::parse_str(&product_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let location_id = Uuid::parse_str(&location_id)
        .map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;

    Ok(PurchaseWithRelations {
        id,
        user_id,
        product_id,
        location_id,
        quantity,
        price,
        purchased_at,
        deleted_at,
        user_name,
        product_brand,
        product_name,
        location_name,
    })
}

/// List purchases with optional filters, including user/product/location names (single JOIN query).
/// Excludes soft-deleted purchases unless `include_deleted`.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn list_with_relations(
    pool: &SqlitePool,
    user_id: Option<Uuid>,
    product_id: Option<Uuid>,
    location_id: Option<Uuid>,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    include_deleted: bool,
) -> Result<Vec<PurchaseWithRelations>, crate::db::DbError> {
    let mut conditions = vec!["1=1".to_string()];
    let mut binds: Vec<String> = Vec::new();

    if let Some(uid) = user_id {
        conditions.push("p.user_id = ?".to_string());
        binds.push(uid.to_string());
    }
    if let Some(pid) = product_id {
        conditions.push("p.product_id = ?".to_string());
        binds.push(pid.to_string());
    }
    if let Some(lid) = location_id {
        conditions.push("p.location_id = ?".to_string());
        binds.push(lid.to_string());
    }
    if let Some(ts) = from_ts {
        conditions.push("p.purchased_at >= ?".to_string());
        binds.push(ts.to_string());
    }
    if let Some(ts) = to_ts {
        conditions.push("p.purchased_at <= ?".to_string());
        binds.push(ts.to_string());
    }
    if !include_deleted {
        conditions.push("p.deleted_at IS NULL".to_string());
    }

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "{PURCHASE_JOIN_SELECT} {PURCHASE_JOIN_FROM} WHERE {where_clause} ORDER BY p.purchased_at DESC"
    );

    let mut query = sqlx::query(&sql);
    for b in &binds {
        query = query.bind(b);
    }
    let rows = query.fetch_all(pool).await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(row_to_purchase_with_relations(&row)?);
    }
    Ok(out)
}

/// Fetch a purchase by id (active only) with user, product, and location names.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id_with_relations(
    pool: &SqlitePool,
    id: Uuid,
) -> Result<Option<PurchaseWithRelations>, crate::db::DbError> {
    let id_str = id.to_string();
    let sql = format!(
        "{PURCHASE_JOIN_SELECT} {PURCHASE_JOIN_FROM} WHERE p.id = ? AND p.deleted_at IS NULL"
    );
    let row = sqlx::query(&sql).bind(&id_str).fetch_optional(pool).await?;

    let Some(row) = row else {
        return Ok(None);
    };
    Ok(Some(row_to_purchase_with_relations(&row)?))
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
