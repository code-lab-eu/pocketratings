//! Integration tests for category DB functions and tree building.

use pocketratings::db;
use pocketratings::domain::category::Category;
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
async fn category_get_by_id_get_parent_get_children_get_all_and_get_tree() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("category_test.db");
    let db_path_str = db_path.to_str().expect("temp path is not valid UTF-8");

    let pool = db::create_pool(db_path_str)
        .await
        .expect("failed to create pool");

    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let root_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO categories (id, parent_id, name, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(root_id.to_string())
    .bind::<Option<String>>(None)
    .bind("Root")
    .bind(1_000_i64)
    .bind(1_000_i64)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert root");

    sqlx::query(
        "INSERT INTO categories (id, parent_id, name, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(child_id.to_string())
    .bind(root_id.to_string())
    .bind("Child")
    .bind(2_000_i64)
    .bind(2_000_i64)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert child");

    let root = db::category::get_by_id(&pool, root_id)
        .await
        .expect("get_by_id root")
        .expect("root should exist");
    assert_eq!(root.name(), "Root");
    assert_eq!(root.parent_id(), None);

    let child = db::category::get_by_id(&pool, child_id)
        .await
        .expect("get_by_id child")
        .expect("child should exist");
    assert_eq!(child.name(), "Child");
    assert_eq!(child.parent_id(), Some(root_id));

    let parent = db::category::get_parent(&pool, child_id)
        .await
        .expect("get_parent")
        .expect("parent should exist");
    assert_eq!(parent.id(), root_id);

    let children = db::category::get_children(&pool, Some(root_id))
        .await
        .expect("get_children");
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id(), child_id);

    let roots = db::category::get_children(&pool, None)
        .await
        .expect("get_children with None");
    assert_eq!(
        roots.len(),
        1,
        "get_children(None) should return only root categories"
    );
    assert_eq!(roots[0].id(), root_id);
    assert_eq!(roots[0].name(), "Root");

    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 2);

    let tree = db::category::Categories::from_list(all, None, None, false);
    assert!(tree.category.is_none());
    assert_eq!(tree.children.len(), 1);
    assert_eq!(
        tree.children[0]
            .category
            .as_ref()
            .map(pocketratings::domain::category::Category::name),
        Some("Root")
    );
    assert_eq!(tree.children[0].children.len(), 1);
    assert_eq!(
        tree.children[0].children[0]
            .category
            .as_ref()
            .map(pocketratings::domain::category::Category::name),
        Some("Child")
    );
}

#[tokio::test]
async fn category_get_children_with_none_returns_only_roots() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_roots.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let roots_empty = db::category::get_children(&pool, None)
        .await
        .expect("get_children(None)");
    assert!(
        roots_empty.is_empty(),
        "get_children(None) on empty DB should return empty list"
    );

    let root_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();
    let now = 1_000_i64;
    let root =
        Category::new(root_id, None, "Root".to_string(), now, now, None).expect("valid category");
    let child = Category::new(
        child_id,
        Some(root_id),
        "Child".to_string(),
        now + 1,
        now + 1,
        None,
    )
    .expect("valid category");
    db::category::insert(&pool, &root)
        .await
        .expect("insert root");
    db::category::insert(&pool, &child)
        .await
        .expect("insert child");

    let roots = db::category::get_children(&pool, None)
        .await
        .expect("get_children(None)");
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].id(), root_id);
    assert_eq!(roots[0].name(), "Root");
    assert_eq!(roots[0].parent_id(), None);
}

#[tokio::test]
async fn category_insert_and_get_by_id_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_insert.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    let now = 1_000_i64;
    let category =
        Category::new(id, None, "Groceries".to_string(), now, now, None).expect("valid category");

    db::category::insert(&pool, &category)
        .await
        .expect("insert");

    let loaded = db::category::get_by_id(&pool, id)
        .await
        .expect("get_by_id")
        .expect("category should exist");
    assert_eq!(loaded.id(), id);
    assert_eq!(loaded.name(), "Groceries");
    assert_eq!(loaded.parent_id(), None);
}

#[tokio::test]
async fn category_update_changes_name() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    let now = 1_000_i64;
    let category =
        Category::new(id, None, "OldName".to_string(), now, now, None).expect("valid category");
    db::category::insert(&pool, &category)
        .await
        .expect("insert");

    let updated = Category::new(id, None, "NewName".to_string(), now, now + 10, None)
        .expect("valid updated category");
    db::category::update(&pool, &updated).await.expect("update");

    let loaded = db::category::get_by_id(&pool, id)
        .await
        .expect("get_by_id")
        .expect("category should exist");
    assert_eq!(loaded.name(), "NewName");
}

#[tokio::test]
async fn category_soft_delete_sets_deleted_at_and_excludes_from_get_all() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_soft_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    let now = 1_000_i64;
    let category =
        Category::new(id, None, "ToDelete".to_string(), now, now, None).expect("valid category");
    db::category::insert(&pool, &category)
        .await
        .expect("insert");

    db::category::soft_delete(&pool, id)
        .await
        .expect("soft_delete");

    let by_id = db::category::get_by_id(&pool, id).await.expect("get_by_id");
    assert!(by_id.is_none());

    let active = db::category::get_all(&pool, false).await.expect("get_all");
    assert!(
        active.is_empty(),
        "soft-deleted category should not be in get_all"
    );

    let with_deleted = db::category::get_all(&pool, true)
        .await
        .expect("get_all with include_deleted");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
async fn category_soft_delete_fails_when_category_has_products() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_soft_delete_products.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = Uuid::new_v4();
    let now = 1_000_i64;
    let category =
        Category::new(cat_id, None, "WithProducts".to_string(), now, now, None).expect("valid");
    db::category::insert(&pool, &category)
        .await
        .expect("insert category");

    // Insert an active product referencing this category.
    let product_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO products (id, category_id, brand, name, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(product_id.to_string())
    .bind(cat_id.to_string())
    .bind("Brand")
    .bind("Product")
    .bind(1_000_i64)
    .bind(1_000_i64)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert product");

    let result = db::category::soft_delete(&pool, cat_id).await;
    assert!(
        result.is_err(),
        "soft_delete should fail when category has products"
    );
}

#[tokio::test]
async fn category_soft_delete_fails_when_category_has_children() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_soft_delete_children.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let parent_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();
    let now = 1_000_i64;
    let parent =
        Category::new(parent_id, None, "Parent".to_string(), now, now, None).expect("valid");
    let child = Category::new(
        child_id,
        Some(parent_id),
        "Child".to_string(),
        now + 1,
        now + 1,
        None,
    )
    .expect("valid");
    db::category::insert(&pool, &parent)
        .await
        .expect("insert parent");
    db::category::insert(&pool, &child)
        .await
        .expect("insert child");

    let result = db::category::soft_delete(&pool, parent_id).await;
    assert!(
        result.is_err(),
        "soft_delete should fail when category has child categories"
    );
}

// --- Category list cache tests (run serially; they enable the cache for the process) ---

#[tokio::test]
#[serial]
async fn category_list_cache_is_warmed_on_first_call_and_used_on_subsequent_calls() {
    db::category::clear_category_list_cache();
    db::category::set_use_category_list_cache_for_test(true);
    let _guard = CacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_cache_warm.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert!(all.is_empty(), "first call: empty DB -> empty list");

    let id = Uuid::new_v4();
    let cached = Category::new(id, None, "CachedOnly".to_string(), 1, 1, None).expect("valid");
    db::category::set_category_list_cache_for_test(Some(vec![cached.clone()]));

    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1, "must return cached list, not DB");
    assert_eq!(all[0].name(), "CachedOnly");

    let with_del = db::category::get_all(&pool, true)
        .await
        .expect("get_all with include_deleted");
    assert_eq!(with_del.len(), 1);
    assert_eq!(with_del[0].name(), "CachedOnly");
}

#[tokio::test]
#[serial]
async fn category_list_cache_invalidated_after_insert() {
    db::category::clear_category_list_cache();
    db::category::set_use_category_list_cache_for_test(true);
    let _guard = CacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_cache_insert.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert!(all.is_empty());
    db::category::set_category_list_cache_for_test(Some(vec![]));
    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert!(all.is_empty(), "cached empty");

    let id = Uuid::new_v4();
    let c = Category::new(id, None, "New".to_string(), 1, 1, None).expect("valid");
    db::category::insert(&pool, &c).await.expect("insert");

    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1, "cache must be invalidated after insert");
    assert_eq!(all[0].name(), "New");
}

#[tokio::test]
#[serial]
async fn category_list_cache_invalidated_after_update() {
    db::category::clear_category_list_cache();
    db::category::set_use_category_list_cache_for_test(true);
    let _guard = CacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_cache_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    let c = Category::new(id, None, "Original".to_string(), 1, 1, None).expect("valid");
    db::category::insert(&pool, &c).await.expect("insert");
    let _ = db::category::get_all(&pool, false).await.expect("get_all");

    let stale =
        Category::new(Uuid::new_v4(), None, "Stale".to_string(), 2, 2, None).expect("valid");
    db::category::set_category_list_cache_for_test(Some(vec![stale]));

    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name(), "Stale", "still cached");

    let updated = Category::new(id, None, "Updated".to_string(), 1, 2, None).expect("valid");
    db::category::update(&pool, &updated).await.expect("update");

    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1, "cache must be invalidated after update");
    assert_eq!(all[0].name(), "Updated");
}

#[tokio::test]
#[serial]
async fn category_list_cache_invalidated_after_soft_delete() {
    db::category::clear_category_list_cache();
    db::category::set_use_category_list_cache_for_test(true);
    let _guard = CacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_cache_soft_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    let c = Category::new(id, None, "ToDelete".to_string(), 1, 1, None).expect("valid");
    db::category::insert(&pool, &c).await.expect("insert");
    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1);

    db::category::soft_delete(&pool, id)
        .await
        .expect("soft_delete");

    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert!(
        all.is_empty(),
        "cache must be invalidated; get_all shows active only"
    );
    let with_del = db::category::get_all(&pool, true)
        .await
        .expect("get_all with include_deleted");
    assert_eq!(with_del.len(), 1);
}

#[tokio::test]
#[serial]
async fn category_list_cache_invalidated_after_hard_delete() {
    db::category::clear_category_list_cache();
    db::category::set_use_category_list_cache_for_test(true);
    let _guard = CacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("category_cache_hard_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    let c = Category::new(id, None, "ToRemove".to_string(), 1, 1, None).expect("valid");
    db::category::insert(&pool, &c).await.expect("insert");
    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1);

    db::category::hard_delete(&pool, id)
        .await
        .expect("hard_delete");

    let all = db::category::get_all(&pool, false).await.expect("get_all");
    assert!(
        all.is_empty(),
        "cache must be invalidated after hard_delete"
    );
}

struct CacheTestGuard;

impl Drop for CacheTestGuard {
    fn drop(&mut self) {
        db::category::clear_category_list_cache();
        db::category::set_use_category_list_cache_for_test(false);
    }
}
