//! Integration tests for category DB functions and tree building.

use pocketratings::db;
use uuid::Uuid;

#[tokio::test]
async fn category_get_by_id_get_parent_get_children_get_all_and_get_tree() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = dir.path().join("category_test.db");
    let db_path_str = db_path
        .to_str()
        .expect("temp path is not valid UTF-8");

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

    let children = db::category::get_children(&pool, root_id)
        .await
        .expect("get_children");
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id(), child_id);

    let all = db::category::get_all(&pool).await.expect("get_all");
    assert_eq!(all.len(), 2);

    let tree = db::category::get_tree(all, None);
    assert!(tree.category.is_none());
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].category.as_ref().map(|c| c.name()), Some("Root"));
    assert_eq!(tree.children[0].children.len(), 1);
    assert_eq!(
        tree.children[0].children[0].category.as_ref().map(|c| c.name()),
        Some("Child")
    );
}
