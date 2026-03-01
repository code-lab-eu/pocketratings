//! Review persistence.
//!
//! Provides DB functions: [`get_by_id`], [`get_by_id_with_relations`], [`list`],
//! [`list_with_relations`], [`insert`], [`update`], [`soft_delete`], and [`hard_delete`].

use std::sync::{OnceLock, RwLock};

use rust_decimal::Decimal;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::review::Review;

/// True when the process is the production binary (`main()` has run). False in test binaries so the
/// cache is off unless a test explicitly enables it via [`set_use_review_list_cache_for_test`].
static RUNNING_AS_PRODUCTION: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn use_cache() -> bool {
    RUNNING_AS_PRODUCTION.load(std::sync::atomic::Ordering::SeqCst)
        || USE_CACHE_IN_TEST.with(|v| v.load(std::sync::atomic::Ordering::SeqCst))
}

/// Call from production `main()` so the review list cache is used. Not called in test binaries.
pub fn set_running_as_production() {
    RUNNING_AS_PRODUCTION.store(true, std::sync::atomic::Ordering::SeqCst);
}

std::thread_local! {
    static USE_CACHE_IN_TEST: std::sync::atomic::AtomicBool = const { std::sync::atomic::AtomicBool::new(false) };
}

/// Enable or disable use of the review list cache in the current thread.
///
/// For use by cache tests only. Thread-local so other tests can run in parallel with cache off.
/// Cache tests should be marked `#[serial_test::serial]` so they don't run in parallel with each
/// other (they share the process-wide cache).
pub fn set_use_review_list_cache_for_test(use_cache: bool) {
    USE_CACHE_IN_TEST.with(|v| v.store(use_cache, std::sync::atomic::Ordering::SeqCst));
}

/// Module-level cache for the full review list with relations (including deleted). Used by
/// `list_with_relations`.
fn review_list_cache() -> &'static RwLock<Option<Vec<ReviewWithRelations>>> {
    static CACHE: OnceLock<RwLock<Option<Vec<ReviewWithRelations>>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(None))
}

fn invalidate_review_list_cache() {
    let _ = review_list_cache().write().map(|mut g| *g = None);
}

/// Clear the review list cache. For use by cache tests so they start from a known state.
pub fn clear_review_list_cache() {
    let _ = review_list_cache().write().map(|mut g| *g = None);
}

/// Set the review list cache to a specific value. For use by cache tests to verify that
/// `list_with_relations` returns cached data when the cache is populated.
pub fn set_review_list_cache_for_test(list: Option<Vec<ReviewWithRelations>>) {
    let _ = review_list_cache().write().map(|mut g| *g = list);
}

/// One review row with joined user and product names for API responses.
#[derive(Debug, Clone)]
pub struct ReviewWithRelations {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_brand: String,
    pub product_name: String,
    pub user_id: Uuid,
    pub user_name: String,
    pub rating: String,
    pub text: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

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

const REVIEW_JOIN_SELECT: &str = "SELECT r.id, r.product_id, r.user_id, r.rating, r.text, r.created_at, r.updated_at, r.deleted_at, \
    u.name AS user_name, p.brand AS product_brand, p.name AS product_name ";
const REVIEW_JOIN_FROM: &str = "FROM reviews r \
    JOIN users u ON r.user_id = u.id \
    JOIN products p ON r.product_id = p.id";

fn row_to_review_with_relations(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<ReviewWithRelations, crate::db::DbError> {
    let id: String = row.get("id");
    let product_id: String = row.get("product_id");
    let user_id: String = row.get("user_id");
    let rating: String = row.get("rating");
    let text: Option<String> = row.get("text");
    let created_at: i64 = row.get("created_at");
    let updated_at: i64 = row.get("updated_at");
    let deleted_at: Option<i64> = row.get("deleted_at");
    let user_name: String = row.get("user_name");
    let product_brand: String = row.get("product_brand");
    let product_name: String = row.get("product_name");

    let id = Uuid::parse_str(&id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let product_id =
        Uuid::parse_str(&product_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let user_id =
        Uuid::parse_str(&user_id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;

    Ok(ReviewWithRelations {
        id,
        product_id,
        product_brand,
        product_name,
        user_id,
        user_name,
        rating,
        text,
        created_at,
        updated_at,
        deleted_at,
    })
}

/// Fetch all reviews with relations from the database (active and soft-deleted). Used to fill the cache.
async fn fetch_all_reviews_with_relations_raw(
    pool: &SqlitePool,
) -> Result<Vec<ReviewWithRelations>, crate::db::DbError> {
    let sql = format!("{REVIEW_JOIN_SELECT} {REVIEW_JOIN_FROM} ORDER BY r.updated_at DESC");
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(row_to_review_with_relations(&row)?);
    }
    Ok(out)
}

fn filter_reviews(
    list: &[ReviewWithRelations],
    product_id: Option<Uuid>,
    user_id: Option<Uuid>,
    include_deleted: bool,
) -> Vec<ReviewWithRelations> {
    list.iter()
        .filter(|r| {
            if !include_deleted && r.deleted_at.is_some() {
                return false;
            }
            if let Some(pid) = product_id
                && r.product_id != pid
            {
                return false;
            }
            if let Some(uid) = user_id
                && r.user_id != uid
            {
                return false;
            }
            true
        })
        .cloned()
        .collect()
}

/// List reviews with optional filters, including user and product names (single JOIN query).
///
/// Excludes soft-deleted unless `include_deleted`. When cache is enabled, uses cached full list
/// and filters in memory.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn list_with_relations(
    pool: &SqlitePool,
    product_id: Option<Uuid>,
    user_id: Option<Uuid>,
    include_deleted: bool,
) -> Result<Vec<ReviewWithRelations>, crate::db::DbError> {
    if use_cache()
        && let Ok(guard) = review_list_cache().read()
        && let Some(ref list) = *guard
    {
        return Ok(filter_reviews(list, product_id, user_id, include_deleted));
    }

    let list = fetch_all_reviews_with_relations_raw(pool).await?;

    if use_cache()
        && let Ok(mut guard) = review_list_cache().write()
    {
        *guard = Some(list.clone());
    }

    Ok(filter_reviews(&list, product_id, user_id, include_deleted))
}

/// Fetch a review by id (active only) with user and product names.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id_with_relations(
    pool: &SqlitePool,
    id: Uuid,
) -> Result<Option<ReviewWithRelations>, crate::db::DbError> {
    let id_str = id.to_string();
    let sql =
        format!("{REVIEW_JOIN_SELECT} {REVIEW_JOIN_FROM} WHERE r.id = ? AND r.deleted_at IS NULL");
    let row = sqlx::query(&sql).bind(&id_str).fetch_optional(pool).await?;

    let Some(row) = row else {
        return Ok(None);
    };
    Ok(Some(row_to_review_with_relations(&row)?))
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
    invalidate_review_list_cache();
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
    invalidate_review_list_cache();
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

    invalidate_review_list_cache();
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

    invalidate_review_list_cache();
    Ok(())
}
