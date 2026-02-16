//! Integration tests for product DB functions.

use pocketratings::db;
use pocketratings::domain::product::Product;
use uuid::Uuid;

#[tokio::test]
async fn product_insert_and_get_by_id_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("product_insert.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = Uuid::new_v4();
    let now = 1_000_i64;
    let category = pocketratings::domain::category::Category::new(
        cat_id,
        None,
        "Groceries".to_string(),
        now,
        now,
        None,
    )
    .expect("valid category");
    db::category::insert(&pool, &category)
        .await
        .expect("insert category");

    let product_id = Uuid::new_v4();
    let product = Product::new(
        product_id,
        cat_id,
        "Acme".to_string(),
        "Widget".to_string(),
        now,
        now,
        None,
    )
    .expect("valid product");
    db::product::insert(&pool, &product).await.expect("insert");

    let loaded = db::product::get_by_id(&pool, product_id)
        .await
        .expect("get_by_id")
        .expect("product should exist");
    assert_eq!(loaded.id(), product_id);
    assert_eq!(loaded.brand(), "Acme");
    assert_eq!(loaded.name(), "Widget");
    assert_eq!(loaded.category_id(), cat_id);
}

#[tokio::test]
async fn product_get_all_and_get_all_by_category_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("product_get_all.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat1 = Uuid::new_v4();
    let cat2 = Uuid::new_v4();
    let now = 1_000_i64;
    let c1 = pocketratings::domain::category::Category::new(
        cat1,
        None,
        "Cat1".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    let c2 = pocketratings::domain::category::Category::new(
        cat2,
        None,
        "Cat2".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::category::insert(&pool, &c1).await.expect("insert");
    db::category::insert(&pool, &c2).await.expect("insert");

    let p1 = Product::new(
        Uuid::new_v4(),
        cat1,
        "B1".to_string(),
        "P1".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    let p2 = Product::new(
        Uuid::new_v4(),
        cat1,
        "B2".to_string(),
        "P2".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    let p3 = Product::new(
        Uuid::new_v4(),
        cat2,
        "B3".to_string(),
        "P3".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::product::insert(&pool, &p1).await.expect("insert");
    db::product::insert(&pool, &p2).await.expect("insert");
    db::product::insert(&pool, &p3).await.expect("insert");

    let all = db::product::get_all(&pool).await.expect("get_all");
    assert_eq!(all.len(), 3);

    let in_cat1 = db::product::get_all_by_category_id(&pool, cat1)
        .await
        .expect("get_all_by_category_id");
    assert_eq!(in_cat1.len(), 2);
    let in_cat2 = db::product::get_all_by_category_id(&pool, cat2)
        .await
        .expect("get_all_by_category_id");
    assert_eq!(in_cat2.len(), 1);
}

#[tokio::test]
async fn product_get_all_with_deleted() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("product_get_all_deleted.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = Uuid::new_v4();
    let now = 1_000_i64;
    let cat = pocketratings::domain::category::Category::new(
        cat_id,
        None,
        "C".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::category::insert(&pool, &cat).await.expect("insert");

    let p = Product::new(
        Uuid::new_v4(),
        cat_id,
        "B".to_string(),
        "P".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::product::insert(&pool, &p).await.expect("insert");

    let active = db::product::get_all(&pool).await.expect("get_all");
    assert_eq!(active.len(), 1);

    db::product::soft_delete(&pool, p.id())
        .await
        .expect("soft_delete");

    let active_after = db::product::get_all(&pool).await.expect("get_all");
    assert!(active_after.is_empty());

    let with_deleted = db::product::get_all_with_deleted(&pool)
        .await
        .expect("get_all_with_deleted");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
async fn product_update_changes_name_and_brand() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("product_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = Uuid::new_v4();
    let now = 1_000_i64;
    let cat = pocketratings::domain::category::Category::new(
        cat_id,
        None,
        "C".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::category::insert(&pool, &cat).await.expect("insert");

    let product_id = Uuid::new_v4();
    let product = Product::new(
        product_id,
        cat_id,
        "OldBrand".to_string(),
        "OldName".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::product::insert(&pool, &product).await.expect("insert");

    let updated = Product::new(
        product_id,
        cat_id,
        "NewBrand".to_string(),
        "NewName".to_string(),
        now,
        now + 10,
        None,
    )
    .expect("valid");
    db::product::update(&pool, &updated).await.expect("update");

    let loaded = db::product::get_by_id(&pool, product_id)
        .await
        .expect("get_by_id")
        .expect("should exist");
    assert_eq!(loaded.brand(), "NewBrand");
    assert_eq!(loaded.name(), "NewName");
}

#[tokio::test]
async fn product_soft_delete_sets_deleted_at_and_excludes_from_get_by_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("product_soft_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = Uuid::new_v4();
    let now = 1_000_i64;
    let cat = pocketratings::domain::category::Category::new(
        cat_id,
        None,
        "C".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::category::insert(&pool, &cat).await.expect("insert");

    let product_id = Uuid::new_v4();
    let product = Product::new(
        product_id,
        cat_id,
        "B".to_string(),
        "P".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::product::insert(&pool, &product).await.expect("insert");

    db::product::soft_delete(&pool, product_id)
        .await
        .expect("soft_delete");

    let by_id = db::product::get_by_id(&pool, product_id)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());

    let with_deleted = db::product::get_all_with_deleted(&pool)
        .await
        .expect("get_all_with_deleted");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
async fn product_hard_delete_removes_row() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("product_hard_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = Uuid::new_v4();
    let now = 1_000_i64;
    let cat = pocketratings::domain::category::Category::new(
        cat_id,
        None,
        "C".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::category::insert(&pool, &cat).await.expect("insert");

    let product_id = Uuid::new_v4();
    let product = Product::new(
        product_id,
        cat_id,
        "B".to_string(),
        "P".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::product::insert(&pool, &product).await.expect("insert");

    db::product::hard_delete(&pool, product_id)
        .await
        .expect("hard_delete");

    let by_id = db::product::get_by_id(&pool, product_id)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());
    let with_deleted = db::product::get_all_with_deleted(&pool)
        .await
        .expect("get_all_with_deleted");
    assert!(with_deleted.is_empty());
}

#[tokio::test]
async fn product_soft_delete_fails_when_product_has_purchases() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("product_soft_delete_purchases.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = Uuid::new_v4();
    let now = 1_000_i64;
    let cat = pocketratings::domain::category::Category::new(
        cat_id,
        None,
        "C".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::category::insert(&pool, &cat)
        .await
        .expect("insert category");

    let product_id = Uuid::new_v4();
    let product = Product::new(
        product_id,
        cat_id,
        "B".to_string(),
        "P".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::product::insert(&pool, &product)
        .await
        .expect("insert product");

    let user_id = Uuid::new_v4();
    let location_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id.to_string())
    .bind("User")
    .bind("u@example.com")
    .bind("hash")
    .bind(now)
    .bind(now)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert user");
    sqlx::query("INSERT INTO locations (id, name, deleted_at) VALUES (?, ?, ?)")
        .bind(location_id.to_string())
        .bind("Store")
        .bind::<Option<i64>>(None)
        .execute(&pool)
        .await
        .expect("insert location");

    let purchase_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO purchases (id, user_id, product_id, location_id, quantity, price, purchased_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(purchase_id.to_string())
    .bind(user_id.to_string())
    .bind(product_id.to_string())
    .bind(location_id.to_string())
    .bind(1_i32)
    .bind("9.99")
    .bind(now)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert purchase");

    let result = db::product::soft_delete(&pool, product_id).await;
    assert!(
        result.is_err(),
        "soft_delete should fail when product has purchases"
    );
}

#[tokio::test]
async fn product_hard_delete_fails_when_product_has_purchases() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("product_hard_delete_purchases.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = Uuid::new_v4();
    let now = 1_000_i64;
    let cat = pocketratings::domain::category::Category::new(
        cat_id,
        None,
        "C".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::category::insert(&pool, &cat)
        .await
        .expect("insert category");

    let product_id = Uuid::new_v4();
    let product = Product::new(
        product_id,
        cat_id,
        "B".to_string(),
        "P".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::product::insert(&pool, &product)
        .await
        .expect("insert product");

    let user_id = Uuid::new_v4();
    let location_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id.to_string())
    .bind("User")
    .bind("u@example.com")
    .bind("hash")
    .bind(now)
    .bind(now)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert user");
    sqlx::query("INSERT INTO locations (id, name, deleted_at) VALUES (?, ?, ?)")
        .bind(location_id.to_string())
        .bind("Store")
        .bind::<Option<i64>>(None)
        .execute(&pool)
        .await
        .expect("insert location");

    sqlx::query(
        "INSERT INTO purchases (id, user_id, product_id, location_id, quantity, price, purchased_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id.to_string())
    .bind(product_id.to_string())
    .bind(location_id.to_string())
    .bind(1_i32)
    .bind("9.99")
    .bind(now)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert purchase");

    let result = db::product::hard_delete(&pool, product_id).await;
    assert!(
        result.is_err(),
        "hard_delete should fail when product has purchases"
    );
}
