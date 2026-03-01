//! Product persistence.
//!
//! Provides DB functions: [`get_by_id`], [`get_by_id_with_relations`], [`get_all`],
//! [`get_all_with_deleted`], [`get_all_by_category_id`], [`get_all_by_category_id_with_deleted`],
//! [`get_all_filtered`], [`list_with_relations`], [`insert`], [`update`], [`soft_delete`], and
//! [`hard_delete`].

use std::sync::{OnceLock, RwLock};

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::product::Product;

/// True when the process is the production binary (`main()` has run). False in test binaries so the
/// cache is off unless a test explicitly enables it via [`set_use_product_list_cache_for_test`].
static RUNNING_AS_PRODUCTION: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn use_cache() -> bool {
    RUNNING_AS_PRODUCTION.load(std::sync::atomic::Ordering::SeqCst)
        || USE_CACHE_IN_TEST.with(|v| v.load(std::sync::atomic::Ordering::SeqCst))
}

/// Call from production `main()` so the product list cache is used. Not called in test binaries.
pub fn set_running_as_production() {
    RUNNING_AS_PRODUCTION.store(true, std::sync::atomic::Ordering::SeqCst);
}

std::thread_local! {
    static USE_CACHE_IN_TEST: std::sync::atomic::AtomicBool = const { std::sync::atomic::AtomicBool::new(false) };
}

/// Enable or disable use of the product list cache in the current thread.
///
/// For use by cache tests only. Thread-local so other tests can run in parallel with cache off.
/// Cache tests should be marked `#[serial_test::serial]` so they don't run in parallel with each
/// other (they share the process-wide cache).
pub fn set_use_product_list_cache_for_test(use_cache: bool) {
    USE_CACHE_IN_TEST.with(|v| v.store(use_cache, std::sync::atomic::Ordering::SeqCst));
}

/// Module-level cache for the full product list with relations (including deleted). Used by
/// `list_with_relations`.
fn product_list_cache() -> &'static RwLock<Option<Vec<ProductWithRelations>>> {
    static CACHE: OnceLock<RwLock<Option<Vec<ProductWithRelations>>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(None))
}

fn invalidate_product_list_cache() {
    let _ = product_list_cache().write().map(|mut g| *g = None);
}

/// Clear the product list cache. For use by cache tests so they start from a known state.
pub fn clear_product_list_cache() {
    let _ = product_list_cache().write().map(|mut g| *g = None);
}

/// Set the product list cache to a specific value. For use by cache tests to verify that
/// `list_with_relations` returns cached data when the cache is populated.
pub fn set_product_list_cache_for_test(list: Option<Vec<ProductWithRelations>>) {
    let _ = product_list_cache().write().map(|mut g| *g = list);
}

/// One product row with joined category name for API responses.
#[derive(Debug, Clone)]
pub struct ProductWithRelations {
    pub id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub brand: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

/// Map a DB row (with `category_name` from JOIN) into [`ProductWithRelations`].
#[allow(clippy::too_many_arguments)]
fn row_to_product_with_relations(
    id: &str,
    category_id: &str,
    category_name: &str,
    brand: &str,
    name: &str,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
) -> Result<ProductWithRelations, crate::db::DbError> {
    let id = Uuid::parse_str(id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let category_id =
        Uuid::parse_str(category_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    Ok(ProductWithRelations {
        id,
        category_id,
        category_name: category_name.to_owned(),
        brand: brand.to_owned(),
        name: name.to_owned(),
        created_at,
        updated_at,
        deleted_at,
    })
}

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

/// Fetch all active products, optionally filtered by category and/or search on name/brand.
///
/// - `category_id`: if `Some`, only products in that category.
/// - `q`: if non-empty after trim, only products where `name` or `brand` contains the string (case-sensitive LIKE).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all_filtered(
    pool: &SqlitePool,
    category_id: Option<Uuid>,
    q: Option<&str>,
) -> Result<Vec<Product>, crate::db::DbError> {
    let search = q.map(str::trim).filter(|s| !s.is_empty());
    match (category_id, search) {
        (None, None) => return get_all(pool).await,
        (Some(cid), None) => return get_all_by_category_id(pool, cid).await,
        (None, Some(term)) => {
            let pattern = format!("%{term}%");
            let rows = sqlx::query(
                "SELECT id, category_id, brand, name, created_at, updated_at, deleted_at FROM products \
                 WHERE deleted_at IS NULL AND (name LIKE ? OR brand LIKE ?)",
            )
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(pool)
            .await?;
            rows_to_products(rows)
        }
        (Some(cid), Some(term)) => {
            let cat_str = cid.to_string();
            let pattern = format!("%{term}%");
            let rows = sqlx::query(
                "SELECT id, category_id, brand, name, created_at, updated_at, deleted_at FROM products \
                 WHERE category_id = ? AND deleted_at IS NULL AND (name LIKE ? OR brand LIKE ?)",
            )
            .bind(&cat_str)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(pool)
            .await?;
            rows_to_products(rows)
        }
    }
}

fn rows_to_products(
    rows: Vec<sqlx::sqlite::SqliteRow>,
) -> Result<Vec<Product>, crate::db::DbError> {
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

const PRODUCT_JOIN_SQL: &str = "SELECT p.id, p.category_id, p.brand, p.name, p.created_at, p.updated_at, p.deleted_at, c.name AS category_name \
    FROM products p JOIN categories c ON p.category_id = c.id ORDER BY p.updated_at DESC";

/// Fetch all products with relations from the database (active and soft-deleted). Used to fill the cache.
async fn fetch_all_products_with_relations_raw(
    pool: &SqlitePool,
) -> Result<Vec<ProductWithRelations>, crate::db::DbError> {
    let rows = sqlx::query(PRODUCT_JOIN_SQL).fetch_all(pool).await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let category_id: String = row.get("category_id");
        let category_name: String = row.get("category_name");
        let brand: String = row.get("brand");
        let name: String = row.get("name");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        out.push(row_to_product_with_relations(
            &id,
            &category_id,
            &category_name,
            &brand,
            &name,
            created_at,
            updated_at,
            deleted_at,
        )?);
    }
    Ok(out)
}

/// Filter products in memory by `category_id`, search term (substring on name and brand, case-insensitive),
/// and `deleted_at` when `include_deleted` is false.
fn filter_products(
    list: &[ProductWithRelations],
    category_id: Option<Uuid>,
    q: Option<&str>,
    include_deleted: bool,
) -> Vec<ProductWithRelations> {
    let search = q.map(str::trim).filter(|s| !s.is_empty());
    let q_lower = search.map(str::to_lowercase);
    list.iter()
        .filter(|p| {
            if !include_deleted && p.deleted_at.is_some() {
                return false;
            }
            if let Some(ref cid) = category_id
                && p.category_id != *cid
            {
                return false;
            }
            if let Some(ref ql) = q_lower {
                let name_ok = p.name.to_lowercase().contains(ql);
                let brand_ok = p.brand.to_lowercase().contains(ql);
                if !name_ok && !brand_ok {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

/// List products with category name, optionally filtered by category and/or search. Excludes
/// soft-deleted unless `include_deleted`. When cache is enabled, uses cached full list and filters
/// in memory.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn list_with_relations(
    pool: &SqlitePool,
    category_id: Option<Uuid>,
    q: Option<&str>,
    include_deleted: bool,
) -> Result<Vec<ProductWithRelations>, crate::db::DbError> {
    if use_cache()
        && let Ok(guard) = product_list_cache().read()
        && let Some(ref list) = *guard
    {
        return Ok(filter_products(list, category_id, q, include_deleted));
    }

    let list = fetch_all_products_with_relations_raw(pool).await?;

    if use_cache()
        && let Ok(mut guard) = product_list_cache().write()
    {
        *guard = Some(list.clone());
    }

    Ok(filter_products(&list, category_id, q, include_deleted))
}

/// Fetch a product by id (active only) with category name.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id_with_relations(
    pool: &SqlitePool,
    id: Uuid,
) -> Result<Option<ProductWithRelations>, crate::db::DbError> {
    let id_str = id.to_string();
    let row = sqlx::query(
        "SELECT p.id, p.category_id, p.brand, p.name, p.created_at, p.updated_at, p.deleted_at, c.name AS category_name \
         FROM products p JOIN categories c ON p.category_id = c.id WHERE p.id = ? AND p.deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let id: String = row.get("id");
    let category_id: String = row.get("category_id");
    let category_name: String = row.get("category_name");
    let brand: String = row.get("brand");
    let name: String = row.get("name");
    let created_at: i64 = row.get("created_at");
    let updated_at: i64 = row.get("updated_at");
    let deleted_at: Option<i64> = row.get("deleted_at");
    Ok(Some(row_to_product_with_relations(
        &id,
        &category_id,
        &category_name,
        &brand,
        &name,
        created_at,
        updated_at,
        deleted_at,
    )?))
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
    invalidate_product_list_cache();
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
    invalidate_product_list_cache();
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

    invalidate_product_list_cache();
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

    invalidate_product_list_cache();
    Ok(())
}
