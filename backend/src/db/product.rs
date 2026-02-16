//! Product persistence.
//!
//! Provides DB functions: [`get_by_id`], [`get_all`], [`get_all_with_deleted`],
//! [`get_all_by_category_id`], [`get_all_by_category_id_with_deleted`], [`insert`],
//! [`update`], [`soft_delete`], and [`hard_delete`].

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::product::Product;

/// Map a DB row into a [`Product`]. Fails on invalid UUID or domain validation.
fn row_to_product(
    id: &str,
    category_id: &str,
    brand: &str,
    name: &str,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
) -> Result<Product, crate::db::DbError> {
    let id = Uuid::parse_str(id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let category_id =
        Uuid::parse_str(category_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    Product::new(
        id,
        category_id,
        brand.to_owned(),
        name.to_owned(),
        created_at,
        updated_at,
        deleted_at,
    )
    .map_err(|e| crate::db::DbError::InvalidData(e.to_string()))
}

/// Fetch a product by id (active only).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Product>, crate::db::DbError> {
    let id_str = id.to_string();
    let row = sqlx::query(
        "SELECT id, category_id, brand, name, created_at, updated_at, deleted_at FROM products WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let id: String = row.get("id");
    let category_id: String = row.get("category_id");
    let brand: String = row.get("brand");
    let name: String = row.get("name");
    let created_at: i64 = row.get("created_at");
    let updated_at: i64 = row.get("updated_at");
    let deleted_at: Option<i64> = row.get("deleted_at");

    let product = row_to_product(
        &id,
        &category_id,
        &brand,
        &name,
        created_at,
        updated_at,
        deleted_at,
    )?;
    Ok(Some(product))
}

/// Fetch all active products.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Product>, crate::db::DbError> {
    let rows = sqlx::query(
        "SELECT id, category_id, brand, name, created_at, updated_at, deleted_at FROM products WHERE deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let category_id: String = row.get("category_id");
        let brand: String = row.get("brand");
        let name: String = row.get("name");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let product = row_to_product(
            &id,
            &category_id,
            &brand,
            &name,
            created_at,
            updated_at,
            deleted_at,
        )?;
        out.push(product);
    }
    Ok(out)
}

/// Fetch all products (active and soft-deleted).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all_with_deleted(pool: &SqlitePool) -> Result<Vec<Product>, crate::db::DbError> {
    let rows = sqlx::query(
        "SELECT id, category_id, brand, name, created_at, updated_at, deleted_at FROM products",
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let category_id: String = row.get("category_id");
        let brand: String = row.get("brand");
        let name: String = row.get("name");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let product = row_to_product(
            &id,
            &category_id,
            &brand,
            &name,
            created_at,
            updated_at,
            deleted_at,
        )?;
        out.push(product);
    }
    Ok(out)
}

/// Fetch all active products in a category.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all_by_category_id(
    pool: &SqlitePool,
    category_id: Uuid,
) -> Result<Vec<Product>, crate::db::DbError> {
    let cat_str = category_id.to_string();
    let rows = sqlx::query(
        "SELECT id, category_id, brand, name, created_at, updated_at, deleted_at FROM products WHERE category_id = ? AND deleted_at IS NULL",
    )
    .bind(&cat_str)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let category_id: String = row.get("category_id");
        let brand: String = row.get("brand");
        let name: String = row.get("name");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let product = row_to_product(
            &id,
            &category_id,
            &brand,
            &name,
            created_at,
            updated_at,
            deleted_at,
        )?;
        out.push(product);
    }
    Ok(out)
}

/// Fetch all products in a category (active and soft-deleted).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all_by_category_id_with_deleted(
    pool: &SqlitePool,
    category_id: Uuid,
) -> Result<Vec<Product>, crate::db::DbError> {
    let cat_str = category_id.to_string();
    let rows = sqlx::query(
        "SELECT id, category_id, brand, name, created_at, updated_at, deleted_at FROM products WHERE category_id = ?",
    )
    .bind(&cat_str)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let category_id: String = row.get("category_id");
        let brand: String = row.get("brand");
        let name: String = row.get("name");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let product = row_to_product(
            &id,
            &category_id,
            &brand,
            &name,
            created_at,
            updated_at,
            deleted_at,
        )?;
        out.push(product);
    }
    Ok(out)
}

/// Insert a product into the database.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure (e.g. foreign key violation).
pub async fn insert(pool: &SqlitePool, product: &Product) -> Result<(), crate::db::DbError> {
    sqlx::query(
        "INSERT INTO products (id, category_id, brand, name, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(product.id().to_string())
    .bind(product.category_id().to_string())
    .bind(product.brand())
    .bind(product.name())
    .bind(product.created_at())
    .bind(product.updated_at())
    .bind(product.deleted_at())
    .execute(pool)
    .await?;
    Ok(())
}

/// Update an existing product.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure.
pub async fn update(pool: &SqlitePool, product: &Product) -> Result<(), crate::db::DbError> {
    sqlx::query(
        "UPDATE products SET category_id = ?, brand = ?, name = ?, created_at = ?, updated_at = ?, deleted_at = ? WHERE id = ?",
    )
    .bind(product.category_id().to_string())
    .bind(product.brand())
    .bind(product.name())
    .bind(product.created_at())
    .bind(product.updated_at())
    .bind(product.deleted_at())
    .bind(product.id().to_string())
    .execute(pool)
    .await?;
    Ok(())
}

/// Check that the product has no purchases. Returns error if it has any.
async fn ensure_no_purchases(
    pool: &SqlitePool,
    product_id: &str,
) -> Result<(), crate::db::DbError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM purchases WHERE product_id = ?")
        .bind(product_id)
        .fetch_one(pool)
        .await?;
    if count > 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "cannot delete product with purchases: {product_id}"
        )));
    }
    Ok(())
}

/// Soft-delete a product by id. Sets `deleted_at` and `updated_at` to the current time.
///
/// Fails if the product has any purchases.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// the product has purchases or no active product exists with the given id.
pub async fn soft_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();
    ensure_no_purchases(pool, &id_str).await?;

    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE products SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(now)
    .bind(&id_str)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "product not found or already deleted: {id_str}"
        )));
    }

    Ok(())
}

/// Hard-delete a product by id (remove the row).
///
/// Fails if the product has any purchases.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// the product has purchases.
pub async fn hard_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();
    ensure_no_purchases(pool, &id_str).await?;

    let result = sqlx::query("DELETE FROM products WHERE id = ?")
        .bind(&id_str)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "product not found: {id_str}"
        )));
    }

    Ok(())
}
