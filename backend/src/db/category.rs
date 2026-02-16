//! Category persistence and tree building.
//!
//! Provides the [`Categories`] tree type, a pure [`get_tree`] that builds a tree from a flat list,
//! and DB functions: [`get_by_id`], [`get_parent`], [`get_children`], [`get_all`],
//! [`get_all_with_deleted`], [`insert`], [`update`], and [`soft_delete`].

use std::collections::HashMap;

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::category::Category;

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
) -> Vec<Categories> {
    let list = match map.get(&parent_id) {
        Some(v) => v.clone(),
        None => return Vec::new(),
    };
    list.into_iter()
        .map(|c| Categories {
            category: Some(c.clone()),
            children: children_for(Some(c.id()), map),
        })
        .collect()
}

/// Build a tree from a flat list of categories.
///
/// If `root` is `None`, returns the full tree with a virtual root (one node with `category: None`
/// and children = root categories). If `root` is `Some(r)`, returns the subtree rooted at `r`.
#[must_use]
pub fn get_tree(flat_list: Vec<Category>, root: Option<Category>) -> Categories {
    let map: HashMap<Option<Uuid>, Vec<Category>> = flat_list.into_iter().fold(
        HashMap::new(),
        |mut acc, c| {
            acc.entry(c.parent_id()).or_default().push(c);
            acc
        },
    );

    root.map_or_else(
        || Categories {
            category: None,
            children: children_for(None, &map),
        },
        |r| Categories {
            category: Some(r.clone()),
            children: children_for(Some(r.id()), &map),
        },
    )
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
        Some(s) => Some(
            Uuid::parse_str(s).map_err(|e| crate::db::DbError::InvalidData(e.to_string()))?,
        ),
        None => None,
    };
    Category::new(id, parent_id, name.to_owned(), created_at, updated_at, deleted_at)
        .map_err(|e| crate::db::DbError::InvalidData(e.to_string()))
}

/// Fetch a category by id (active only).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Category>, crate::db::DbError> {
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

/// Fetch direct children of a category (active only).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_children(
    pool: &SqlitePool,
    parent_id: Uuid,
) -> Result<Vec<Category>, crate::db::DbError> {
    let parent_str = parent_id.to_string();
    let rows = sqlx::query(
        "SELECT id, parent_id, name, created_at, updated_at, deleted_at FROM categories WHERE parent_id = ? AND deleted_at IS NULL",
    )
    .bind(&parent_str)
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

/// Fetch all active categories (flat list). Use with [`get_tree`] for the full tree.
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Category>, crate::db::DbError> {
    let rows = sqlx::query(
        "SELECT id, parent_id, name, created_at, updated_at, deleted_at FROM categories WHERE deleted_at IS NULL",
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

/// Fetch all categories (active and soft-deleted).
///
/// # Errors
///
/// Returns [`crate::db::DbError`] on query or row mapping failure.
pub async fn get_all_with_deleted(pool: &SqlitePool) -> Result<Vec<Category>, crate::db::DbError> {
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_category(id: Uuid, parent_id: Option<Uuid>, name: &str) -> Category {
        Category::new(id, parent_id, name.to_owned(), 1, 1, None).expect("valid")
    }

    #[test]
    fn get_tree_empty_list_root_none() {
        let tree = get_tree(Vec::new(), None);
        assert!(tree.category.is_none());
        assert!(tree.children.is_empty());
    }

    #[test]
    fn get_tree_two_roots_root_none() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let flat = vec![
            make_category(id1, None, "A"),
            make_category(id2, None, "B"),
        ];
        let tree = get_tree(flat, None);
        assert!(tree.category.is_none());
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].category.as_ref().map(|c| c.name()), Some("A"));
        assert_eq!(tree.children[1].category.as_ref().map(|c| c.name()), Some("B"));
        assert!(tree.children[0].children.is_empty());
        assert!(tree.children[1].children.is_empty());
    }

    #[test]
    fn get_tree_one_root_one_child_root_none() {
        let root_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();
        let flat = vec![
            make_category(root_id, None, "Root"),
            make_category(child_id, Some(root_id), "Child"),
        ];
        let tree = get_tree(flat, None);
        assert!(tree.category.is_none());
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].category.as_ref().map(|c| c.name()), Some("Root"));
        assert_eq!(tree.children[0].children.len(), 1);
        assert_eq!(
            tree.children[0].children[0].category.as_ref().map(|c| c.name()),
            Some("Child")
        );
        assert!(tree.children[0].children[0].children.is_empty());
    }

    #[test]
    fn get_tree_one_root_one_child_root_some() {
        let root_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();
        let root_cat = make_category(root_id, None, "Root");
        let flat = vec![
            root_cat.clone(),
            make_category(child_id, Some(root_id), "Child"),
        ];
        let tree = get_tree(flat, Some(root_cat));
        assert_eq!(tree.category.as_ref().map(|c| c.name()), Some("Root"));
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].category.as_ref().map(|c| c.name()), Some("Child"));
        assert!(tree.children[0].children.is_empty());
    }

    #[test]
    fn get_tree_empty_list_root_some() {
        let root_cat = make_category(Uuid::new_v4(), None, "Only");
        let tree = get_tree(Vec::new(), Some(root_cat.clone()));
        assert_eq!(tree.category.as_ref().map(|c| c.id()), Some(root_cat.id()));
        assert!(tree.children.is_empty());
    }
}
