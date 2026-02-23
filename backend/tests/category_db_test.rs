//! Integration tests for category DB functions and tree building.

use pocketratings::db;
use pocketratings::domain::category::Category;
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

    let all = db::category::get_all(&pool).await.expect("get_all");
    assert_eq!(all.len(), 2);

    let tree = db::category::get_tree(all, None);
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

    let active = db::category::get_all(&pool).await.expect("get_all");
    assert!(
        active.is_empty(),
        "soft-deleted category should not be in get_all"
    );

    let with_deleted = db::category::get_all_with_deleted(&pool)
        .await
        .expect("get_all_with_deleted");
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
