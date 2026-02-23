//! Category persistence and tree building.
//!
//! Provides the [`Categories`] tree type, [`Categories::from_list`] that builds a tree from a flat
//! list, and DB functions: [`get_by_id`], [`get_parent`], [`get_children`], [`get_all`],
//! [`get_all_with_deleted`], [`insert`], [`update`], and [`soft_delete`].

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::category::Category;

/// True when the process is the production binary (`main()` has run). False in test binaries so the
/// cache is off unless a test explicitly enables it via [`set_use_category_list_cache_for_test`].
static RUNNING_AS_PRODUCTION: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn use_cache() -> bool {
    RUNNING_AS_PRODUCTION.load(std::sync::atomic::Ordering::SeqCst)
        || USE_CACHE_IN_TEST.with(|v| v.load(std::sync::atomic::Ordering::SeqCst))
}

/// Call from production `main()` so the category list cache is used. Not called in test binaries.
pub fn set_running_as_production() {
    RUNNING_AS_PRODUCTION.store(true, std::sync::atomic::Ordering::SeqCst);
}

// Thread-local: when true in this thread, get_all/get_all_with_deleted use the cache. Lets cache
// tests enable the cache without affecting parallel tests (they see default false).
std::thread_local! {
    static USE_CACHE_IN_TEST: std::sync::atomic::AtomicBool = const { std::sync::atomic::AtomicBool::new(false) };
}

/// Enable or disable use of the category list cache in the current thread.
///
/// For use by cache tests only. Thread-local so other tests can run in parallel with cache off.
/// Cache tests should be marked `#[serial_test::serial]` so they don't run in parallel with each
/// other (they share the process-wide cache).
pub fn set_use_category_list_cache_for_test(use_cache: bool) {
    USE_CACHE_IN_TEST.with(|v| v.store(use_cache, std::sync::atomic::Ordering::SeqCst));
}

/// Module-level cache for the full category list (including deleted). Used by `get_all` and
/// `get_all_with_deleted`.
///
/// **Disabled in test builds by default:** The cache is a single process-wide static. Unrelated
/// tests run in parallel with their own DBs; we bypass the cache so they don't see each other's
/// data. Cache tests enable it via [`set_use_category_list_cache_for_test`] and run under
/// `#[serial]` so only one runs at a time.
fn category_list_cache() -> &'static RwLock<Option<Vec<Category>>> {
    static CACHE: OnceLock<RwLock<Option<Vec<Category>>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(None))
}

fn invalidate_category_list_cache() {
    let _ = category_list_cache().write().map(|mut g| *g = None);
}

/// Clear the category list cache. For use by cache tests so they start from a known state.
pub fn clear_category_list_cache() {
    let _ = category_list_cache().write().map(|mut g| *g = None);
}

/// Set the category list cache to a specific value. For use by cache tests to verify that
/// `get_all` / `get_all_with_deleted` return cached data when the cache is populated.
pub fn set_category_list_cache_for_test(list: Option<Vec<Category>>) {
    let _ = category_list_cache().write().map(|mut g| *g = list);
}

/// Fetch all categories from the database (active and soft-deleted). Used to fill the cache.
async fn fetch_all_categories_raw(pool: &SqlitePool) -> Result<Vec<Category>, crate::db::DbError> {
    let rows = sqlx::query(
        "SELECT id, parent_id, name, created_at, updated_at, deleted_at FROM categories",
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let parent_id: Option<String> = row.get("parent_id");
        let name: String = row.get("name");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let category = row_to_category(
            &id,
            parent_id.as_deref(),
            &name,
            created_at,
            updated_at,
            deleted_at,
        )?;
        out.push(category);
    }
    Ok(out)
}

/// A tree of categories: a node (optionally a category) and its children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Categories {
    /// The category at this node, or `None` for a virtual root.
    pub category: Option<Category>,
    /// Child nodes.
    pub children: Vec<Self>,
}

fn children_for(
    parent_id: Option<Uuid>,
    map: &HashMap<Option<Uuid>, Vec<Category>>,
    remaining_depth: Option<u8>,
) -> Vec<Categories> {
    if remaining_depth == Some(0) {
        return Vec::new();
    }
    let list = match map.get(&parent_id) {
        Some(v) => v.clone(),
        None => return Vec::new(),
    };
    let next_depth = remaining_depth.map(|n| n.saturating_sub(1));
    list.into_iter()
        .map(|c| Categories {
            category: Some(c.clone()),
            children: children_for(Some(c.id()), map, next_depth),
        })
        .collect()
}

impl Categories {
    /// Build a tree from a flat list of categories.
    ///
    /// If `root` is `None`, returns the full tree with a virtual root (one node with `category:
    /// None` and children = root categories). If `root` is `Some(r)`, returns the subtree rooted
    /// at `r`.
    ///
    /// When `include_deleted` is `false`, categories with `deleted_at.is_some()` are filtered out
    /// before building the tree. When `true`, the full list is used.
    ///
    /// When `depth` is `Some(n)`, only up to `n` levels of children are included (e.g. `Some(1)`
    /// = roots only at virtual root, or root + direct children when `root` is `Some`). When
    /// `None`, the tree is unbounded.
    #[must_use]
    pub fn from_list(
        flat_list: Vec<Category>,
        root: Option<Category>,
        depth: Option<u8>,
        include_deleted: bool,
    ) -> Self {
        let list: Vec<Category> = if include_deleted {
            flat_list
        } else {
            flat_list.into_iter().filter(Category::is_active).collect()
        };
        let map: HashMap<Option<Uuid>, Vec<Category>> =
            list.into_iter().fold(HashMap::new(), |mut acc, c| {
                acc.entry(c.parent_id()).or_default().push(c);
                acc
            });

        root.map_or_else(
            || Self {
                category: None,
                children: children_for(None, &map, depth),
            },
            |r| Self {
                category: Some(r.clone()),
                children: children_for(Some(r.id()), &map, depth),
            },
        )
    }
}

/// Map a DB row into a [`Category`]. Fails on invalid UUID or domain validation.
fn row_to_category(
    id: &str,
    parent_id: Option<&str>,
    name: &str,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
) -> Result<Category, crate::db::DbError> {
    let id = Uuid::parse_str(id).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?;
    let parent_id = match parent_id {
        Some(s) => {
            Some(Uuid::parse_str(s).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?)
        }
        None => None,
    };
    Category::new(
        id,
        parent_id,
        name.to_owned(),
        created_at,
        updated_at,
        deleted_at,
    )
    .map_err(|e| crate::db::DbError::InvalidData(e.to_string()))
}

/// Fetch a category by id (active only).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id(
    pool: &SqlitePool,
    id: Uuid,
) -> Result<Option<Category>, crate::db::DbError> {
    let id_str = id.to_string();
    let row = sqlx::query(
        "SELECT id, parent_id, name, created_at, updated_at, deleted_at FROM categories WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let id: String = row.get("id");
    let parent_id: Option<String> = row.get("parent_id");
    let name: String = row.get("name");
    let created_at: i64 = row.get("created_at");
    let updated_at: i64 = row.get("updated_at");
    let deleted_at: Option<i64> = row.get("deleted_at");

    let category = row_to_category(
        &id,
        parent_id.as_deref(),
        &name,
        created_at,
        updated_at,
        deleted_at,
    )?;
    Ok(Some(category))
}

/// Fetch the parent of a category (active only).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_parent(
    pool: &SqlitePool,
    category_id: Uuid,
) -> Result<Option<Category>, crate::db::DbError> {
    let Some(cat) = get_by_id(pool, category_id).await? else {
        return Ok(None);
    };
    let Some(pid) = cat.parent_id() else {
        return Ok(None);
    };
    get_by_id(pool, pid).await
}

/// Fetch direct children of a category, or root categories when `parent_id` is `None` (active only).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_children(
    pool: &SqlitePool,
    parent_id: Option<Uuid>,
) -> Result<Vec<Category>, crate::db::DbError> {
    let rows = match parent_id {
        Some(pid) => {
            let parent_str = pid.to_string();
            sqlx::query(
                "SELECT id, parent_id, name, created_at, updated_at, deleted_at FROM categories WHERE parent_id = ? AND deleted_at IS NULL",
            )
            .bind(&parent_str)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query(
                "SELECT id, parent_id, name, created_at, updated_at, deleted_at FROM categories WHERE parent_id IS NULL AND deleted_at IS NULL",
            )
            .fetch_all(pool)
            .await?
        }
    };

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let parent_id: Option<String> = row.get("parent_id");
        let name: String = row.get("name");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.get("deleted_at");
        let category = row_to_category(
            &id,
            parent_id.as_deref(),
            &name,
            created_at,
            updated_at,
            deleted_at,
        )?;
        out.push(category);
    }
    Ok(out)
}

/// Fetch all active categories (flat list). Use with [`Categories::from_list`] for the full tree.
///
/// When not in test, results are cached in memory; the cache is invalidated on any category
/// insert, update, or delete.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Category>, crate::db::DbError> {
    if use_cache()
        && let Ok(guard) = category_list_cache().read()
        && let Some(ref list) = *guard
    {
        return Ok(list.iter().filter(|c| c.is_active()).cloned().collect());
    }

    let list = fetch_all_categories_raw(pool).await?;

    if use_cache()
        && let Ok(mut guard) = category_list_cache().write()
    {
        *guard = Some(list.clone());
    }

    Ok(list.into_iter().filter(Category::is_active).collect())
}

/// Fetch all categories (active and soft-deleted).
///
/// When not in test, results are cached in memory; the cache is invalidated on any category
/// insert, update, or delete.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all_with_deleted(pool: &SqlitePool) -> Result<Vec<Category>, crate::db::DbError> {
    if use_cache()
        && let Ok(guard) = category_list_cache().read()
        && let Some(ref list) = *guard
    {
        return Ok(list.clone());
    }

    let list = fetch_all_categories_raw(pool).await?;

    if use_cache()
        && let Ok(mut guard) = category_list_cache().write()
    {
        *guard = Some(list.clone());
    }

    Ok(list)
}

/// Insert a category into the database.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure (e.g. duplicate name under parent).
pub async fn insert(pool: &SqlitePool, category: &Category) -> Result<(), crate::db::DbError> {
    sqlx::query(
        "INSERT INTO categories (id, parent_id, name, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(category.id().to_string())
    .bind(category.parent_id().map(|id| id.to_string()))
    .bind(category.name())
    .bind(category.created_at())
    .bind(category.updated_at())
    .bind(category.deleted_at())
    .execute(pool)
    .await?;
    invalidate_category_list_cache();
    Ok(())
}

/// Update an existing category's parent, name, and timestamps.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure (e.g. duplicate name under parent).
pub async fn update(pool: &SqlitePool, category: &Category) -> Result<(), crate::db::DbError> {
    sqlx::query(
        "UPDATE categories SET parent_id = ?, name = ?, created_at = ?, updated_at = ?, deleted_at = ? WHERE id = ?",
    )
    .bind(category.parent_id().map(|id| id.to_string()))
    .bind(category.name())
    .bind(category.created_at())
    .bind(category.updated_at())
    .bind(category.deleted_at())
    .bind(category.id().to_string())
    .execute(pool)
    .await?;
    invalidate_category_list_cache();
    Ok(())
}

/// Soft-delete a category by id. Sets `deleted_at` and `updated_at` to the current time.
///
/// Fails if there are any active child categories or active products belonging to this category.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// the category has child categories, has products, or no active category exists with the given id.
pub async fn soft_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();

    // Check for active child categories.
    let child_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM categories WHERE parent_id = ? AND deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_one(pool)
    .await?;
    if child_count > 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "cannot delete category with child categories: {id_str}"
        )));
    }

    // Check for active products that reference this category.
    let product_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM products WHERE category_id = ? AND deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_one(pool)
    .await?;
    if product_count > 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "cannot delete category with active products: {id_str}"
        )));
    }

    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE categories SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(now)
    .bind(&id_str)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "category not found or already deleted: {id_str}"
        )));
    }

    invalidate_category_list_cache();
    Ok(())
}

/// Hard-delete a category by id (remove the row).
///
/// Fails if there are any active child categories or active products belonging to this category.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query failure, or [`crate::db::DbError::InvalidData`] if
/// the category has child categories, has products, or no active category exists with the given id.
pub async fn hard_delete(pool: &SqlitePool, id: Uuid) -> Result<(), crate::db::DbError> {
    let id_str = id.to_string();

    // Same checks as soft_delete.
    let child_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM categories WHERE parent_id = ? AND deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_one(pool)
    .await?;
    if child_count > 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "cannot delete category with child categories: {id_str}"
        )));
    }

    let product_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM products WHERE category_id = ? AND deleted_at IS NULL",
    )
    .bind(&id_str)
    .fetch_one(pool)
    .await?;
    if product_count > 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "cannot delete category with active products: {id_str}"
        )));
    }

    let result = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(&id_str)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::db::DbError::InvalidData(format!(
            "category not found: {id_str}"
        )));
    }

    invalidate_category_list_cache();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_category(id: Uuid, parent_id: Option<Uuid>, name: &str) -> Category {
        Category::new(id, parent_id, name.to_owned(), 1, 1, None).expect("valid")
    }

    #[test]
    fn from_list_empty_list_root_none() {
        let tree = Categories::from_list(Vec::new(), None, None, false);
        assert!(tree.category.is_none());
        assert!(tree.children.is_empty());
    }

    #[test]
    fn from_list_two_roots_root_none() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let flat = vec![make_category(id1, None, "A"), make_category(id2, None, "B")];
        let tree = Categories::from_list(flat, None, None, false);
        assert!(tree.category.is_none());
        assert_eq!(tree.children.len(), 2);
        assert_eq!(
            tree.children[0]
                .category
                .as_ref()
                .map(crate::domain::category::Category::name),
            Some("A")
        );
        assert_eq!(
            tree.children[1]
                .category
                .as_ref()
                .map(crate::domain::category::Category::name),
            Some("B")
        );
        assert!(tree.children[0].children.is_empty());
        assert!(tree.children[1].children.is_empty());
    }

    #[test]
    fn from_list_one_root_one_child_root_none() {
        let root_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();
        let flat = vec![
            make_category(root_id, None, "Root"),
            make_category(child_id, Some(root_id), "Child"),
        ];
        let tree = Categories::from_list(flat, None, None, false);
        assert!(tree.category.is_none());
        assert_eq!(tree.children.len(), 1);
        assert_eq!(
            tree.children[0]
                .category
                .as_ref()
                .map(crate::domain::category::Category::name),
            Some("Root")
        );
        assert_eq!(tree.children[0].children.len(), 1);
        assert_eq!(
            tree.children[0].children[0]
                .category
                .as_ref()
                .map(crate::domain::category::Category::name),
            Some("Child")
        );
        assert!(tree.children[0].children[0].children.is_empty());
    }

    #[test]
    fn from_list_depth_1_excludes_children() {
        let root_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();
        let flat = vec![
            make_category(root_id, None, "Root"),
            make_category(child_id, Some(root_id), "Child"),
        ];
        let tree = Categories::from_list(flat, None, Some(1), false);
        assert!(tree.category.is_none());
        assert_eq!(tree.children.len(), 1);
        assert_eq!(
            tree.children[0]
                .category
                .as_ref()
                .map(crate::domain::category::Category::name),
            Some("Root")
        );
        assert!(
            tree.children[0].children.is_empty(),
            "depth=1 should not include grandchildren"
        );
    }

    #[test]
    fn from_list_include_deleted_false_filters_deleted() {
        let root_id = Uuid::new_v4();
        let deleted_id = Uuid::new_v4();
        let root = make_category(root_id, None, "Root");
        let deleted_cat =
            Category::new(deleted_id, None, "Deleted".to_string(), 1, 1, Some(99)).expect("valid");
        let flat = vec![root, deleted_cat];
        let tree = Categories::from_list(flat, None, None, false);
        assert_eq!(tree.children.len(), 1);
        assert_eq!(
            tree.children[0]
                .category
                .as_ref()
                .map(crate::domain::category::Category::name),
            Some("Root")
        );
    }

    #[test]
    fn from_list_include_deleted_true_includes_deleted() {
        let root_id = Uuid::new_v4();
        let deleted_id = Uuid::new_v4();
        let deleted_cat =
            Category::new(deleted_id, None, "Deleted".to_string(), 1, 1, Some(99)).expect("valid");
        let flat = vec![make_category(root_id, None, "Root"), deleted_cat];
        let tree = Categories::from_list(flat, None, None, true);
        assert_eq!(tree.children.len(), 2);
    }

    #[test]
    fn from_list_one_root_one_child_root_some() {
        let root_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();
        let root_cat = make_category(root_id, None, "Root");
        let flat = vec![
            root_cat.clone(),
            make_category(child_id, Some(root_id), "Child"),
        ];
        let tree = Categories::from_list(flat, Some(root_cat), None, false);
        assert_eq!(
            tree.category
                .as_ref()
                .map(crate::domain::category::Category::name),
            Some("Root")
        );
        assert_eq!(tree.children.len(), 1);
        assert_eq!(
            tree.children[0]
                .category
                .as_ref()
                .map(crate::domain::category::Category::name),
            Some("Child")
        );
        assert!(tree.children[0].children.is_empty());
    }

    #[test]
    fn from_list_empty_list_root_some() {
        let root_cat = make_category(Uuid::new_v4(), None, "Only");
        let tree = Categories::from_list(Vec::new(), Some(root_cat.clone()), None, false);
        assert_eq!(
            tree.category
                .as_ref()
                .map(crate::domain::category::Category::id),
            Some(root_cat.id())
        );
        assert!(tree.children.is_empty());
    }
}
