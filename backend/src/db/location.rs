//! Location persistence.
//!
//! Provides DB functions: [`get_by_id`], [`get_all`], [`insert`], [`update`],
//! [`soft_delete`], and [`hard_delete`].
//!
//! When running as production, [`get_all`] results are cached in memory (full list
//! including deleted); when `include_deleted` is `false` the result is filtered on
//! read. The cache is invalidated on any insert, update, `soft_delete`, or
//! `hard_delete`.

use std::sync::{OnceLock, RwLock};

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::location::Location;

/// True when the process is the production binary (`main()` has run). False in test
/// binaries so the cache is off unless a test explicitly enables it via
/// [`set_use_location_list_cache_for_test`].
static RUNNING_AS_PRODUCTION: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn use_cache() -> bool {
    RUNNING_AS_PRODUCTION.load(std::sync::atomic::Ordering::SeqCst)
        || USE_CACHE_IN_TEST.with(|v| v.load(std::sync::atomic::Ordering::SeqCst))
}

/// Call from production `main()` so the location list cache is used. Not called in
/// test binaries.
pub fn set_running_as_production() {
    RUNNING_AS_PRODUCTION.store(true, std::sync::atomic::Ordering::SeqCst);
}

// Thread-local: when true in this thread, get_all(..., include_deleted) uses the
// cache. Lets cache tests enable the cache without affecting parallel tests.
std::thread_local! {
    static USE_CACHE_IN_TEST: std::sync::atomic::AtomicBool = const { std::sync::atomic::AtomicBool::new(false) };
}

/// Enable or disable use of the location list cache in the current thread.
///
/// For use by cache tests only. Thread-local so other tests can run in parallel
/// with cache off. Cache tests should be marked `#[serial_test::serial]` so they
/// don't run in parallel with each other (they share the process-wide cache).
pub fn set_use_location_list_cache_for_test(use_cache: bool) {
    USE_CACHE_IN_TEST.with(|v| v.store(use_cache, std::sync::atomic::Ordering::SeqCst));
}

/// Module-level cache for the full location list (including deleted). Used by
/// `get_all(..., include_deleted)`.
///
/// **Disabled in test builds by default:** The cache is a single process-wide
/// static. Unrelated tests run in parallel with their own DBs; we bypass the cache
/// so they don't see each other's data. Cache tests enable it via
/// [`set_use_location_list_cache_for_test`] and run under `#[serial]` so only one
/// runs at a time.
fn location_list_cache() -> &'static RwLock<Option<Vec<Location>>> {
    static CACHE: OnceLock<RwLock<Option<Vec<Location>>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(None))
}

fn invalidate_location_list_cache() {
    let _ = location_list_cache().write().map(|mut g| *g = None);
}

/// Clear the location list cache. For use by cache tests so they start from a
/// known state.
pub fn clear_location_list_cache() {
    let _ = location_list_cache().write().map(|mut g| *g = None);
}

/// Set the location list cache to a specific value. For use by cache tests that
/// need to inject stale data to verify invalidation.
pub fn set_location_list_cache_for_test(list: Option<Vec<Location>>) {
    let _ = location_list_cache().write().map(|mut g| *g = list);
}

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

/// Fetch a location by id.
///
/// When `include_deleted` is `false`, only active locations (`deleted_at` IS NULL) are returned.
/// When `true`, the row may be soft-deleted.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id(
    pool: &SqlitePool,
    id: Uuid,
    include_deleted: bool,
) -> Result<Option<Location>, crate::db::DbError> {
    let id_str = id.to_string();
    let row = if include_deleted {
        sqlx::query("SELECT id, name, deleted_at FROM locations WHERE id = ?")
            .bind(&id_str)
            .fetch_optional(pool)
            .await?
    } else {
        sqlx::query(
            "SELECT id, name, deleted_at FROM locations WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(&id_str)
        .fetch_optional(pool)
        .await?
    };

    let Some(row) = row else {
        return Ok(None);
    };

    let id: String = row.get("id");
    let name: String = row.get("name");
    let deleted_at: Option<i64> = row.get("deleted_at");

    let location = row_to_location(&id, &name, deleted_at)?;
    Ok(Some(location))
}

/// Fetch all locations from the database (active and soft-deleted). Used to fill
/// the cache.
async fn fetch_all_locations_raw(pool: &SqlitePool) -> Result<Vec<Location>, crate::db::DbError> {
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

/// Fetch all locations (flat list).
///
/// When `include_deleted` is `false`, only active (non-deleted) locations are
/// returned. When `true`, soft-deleted locations are included. The cache always
/// stores the full list (including deleted); when `include_deleted` is `false`
/// the result is filtered on read.
///
/// When not in test, results are cached in memory; the cache is invalidated on any
/// location insert, update, or delete.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all(
    pool: &SqlitePool,
    include_deleted: bool,
) -> Result<Vec<Location>, crate::db::DbError> {
    if use_cache()
        && let Ok(guard) = location_list_cache().read()
        && let Some(ref list) = *guard
    {
        return Ok(if include_deleted {
            list.clone()
        } else {
            list.iter().filter(|l| l.is_active()).cloned().collect()
        });
    }

    let list = fetch_all_locations_raw(pool).await?;

    if use_cache()
        && let Ok(mut guard) = location_list_cache().write()
    {
        *guard = Some(list.clone());
    }

    Ok(if include_deleted {
        list
    } else {
        list.into_iter().filter(Location::is_active).collect()
    })
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
    invalidate_location_list_cache();
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
    invalidate_location_list_cache();
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

    invalidate_location_list_cache();
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

    invalidate_location_list_cache();
    Ok(())
}
