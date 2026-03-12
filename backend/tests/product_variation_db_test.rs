//! Integration tests for `product_variation` DB functions.

use pocketratings::db;
use pocketratings::domain::category::Category;
use pocketratings::domain::location::Location;
use pocketratings::domain::product::Product;
use pocketratings::domain::product_variation::ProductVariation;
use pocketratings::domain::purchase::Purchase;
use pocketratings::domain::user::User;
use rust_decimal::Decimal;
use uuid::Uuid;

async fn setup_pool() -> (tempfile::TempDir, sqlx::SqlitePool) {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("product_variation.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");
    (dir, pool)
}

async fn insert_category_and_product(pool: &sqlx::SqlitePool) -> (Uuid, Uuid) {
    let now = 1_000_i64;
    let cat_id = Uuid::new_v4();
    let cat = Category::new(cat_id, None, "Cat".to_string(), now, now, None).expect("valid");
    db::category::insert(pool, &cat)
        .await
        .expect("insert category");

    let product_id = Uuid::new_v4();
    let product = Product::new(
        product_id,
        cat_id,
        "Brand".to_string(),
        "Product".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::product::insert(pool, &product)
        .await
        .expect("insert product");
    (cat_id, product_id)
}

#[tokio::test]
async fn product_variation_insert_and_get_by_id_roundtrip() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var =
        ProductVariation::new(var_id, product_id, "500 g", "grams", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert");

    let loaded = db::product_variation::get_by_id(&pool, var_id, false)
        .await
        .expect("get_by_id")
        .expect("variation should exist");
    assert_eq!(loaded.id(), var_id);
    assert_eq!(loaded.product_id(), product_id);
    assert_eq!(loaded.label(), "500 g");
    assert_eq!(loaded.unit(), "grams");
    assert!(loaded.is_active());
}

#[tokio::test]
async fn product_variation_get_by_id_returns_none_for_nonexistent() {
    let (_dir, pool) = setup_pool().await;
    let id = Uuid::new_v4();
    let result = db::product_variation::get_by_id(&pool, id, false)
        .await
        .expect("get_by_id");
    assert!(result.is_none());
}

#[tokio::test]
async fn product_variation_get_by_id_excludes_soft_deleted_when_include_deleted_false() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var = ProductVariation::new(var_id, product_id, "", "none", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert");
    db::product_variation::soft_delete(&pool, var_id)
        .await
        .expect("soft_delete");

    let result = db::product_variation::get_by_id(&pool, var_id, false)
        .await
        .expect("get_by_id");
    assert!(result.is_none());
}

#[tokio::test]
async fn product_variation_get_by_id_includes_soft_deleted_when_include_deleted_true() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var =
        ProductVariation::new(var_id, product_id, "Large", "other", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert");
    db::product_variation::soft_delete(&pool, var_id)
        .await
        .expect("soft_delete");

    let loaded = db::product_variation::get_by_id(&pool, var_id, true)
        .await
        .expect("get_by_id")
        .expect("should exist when include_deleted");
    assert_eq!(loaded.id(), var_id);
    assert_eq!(loaded.label(), "Large");
    assert!(!loaded.is_active());
}

#[tokio::test]
async fn product_variation_list_by_product_id_empty_then_with_variations() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let list = db::product_variation::list_by_product_id(&pool, product_id, false)
        .await
        .expect("list");
    assert!(list.is_empty());

    let now = 1_000_i64;
    let var1_id = Uuid::new_v4();
    let var1 = ProductVariation::new(var1_id, product_id, "Small", "none", now, now + 1, None)
        .expect("valid");
    db::product_variation::insert(&pool, &var1)
        .await
        .expect("insert");
    let var2_id = Uuid::new_v4();
    let var2 = ProductVariation::new(var2_id, product_id, "Large", "grams", now, now + 2, None)
        .expect("valid");
    db::product_variation::insert(&pool, &var2)
        .await
        .expect("insert");

    let list = db::product_variation::list_by_product_id(&pool, product_id, false)
        .await
        .expect("list");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].label(), "Small");
    assert_eq!(list[1].label(), "Large");
}

#[tokio::test]
async fn product_variation_list_by_product_id_include_deleted_filters_soft_deleted() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var1_id = Uuid::new_v4();
    let var1 =
        ProductVariation::new(var1_id, product_id, "A", "none", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var1)
        .await
        .expect("insert");
    db::product_variation::soft_delete(&pool, var1_id)
        .await
        .expect("soft_delete");
    let var2_id = Uuid::new_v4();
    let var2 =
        ProductVariation::new(var2_id, product_id, "B", "none", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var2)
        .await
        .expect("insert");

    let active = db::product_variation::list_by_product_id(&pool, product_id, false)
        .await
        .expect("list");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].label(), "B");

    let with_deleted = db::product_variation::list_by_product_id(&pool, product_id, true)
        .await
        .expect("list");
    assert_eq!(with_deleted.len(), 2);
}

#[tokio::test]
async fn product_variation_update_changes_label_and_unit() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var =
        ProductVariation::new(var_id, product_id, "Old", "none", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert");

    let updated = ProductVariation::new(
        var_id,
        product_id,
        "New label",
        "milliliters",
        now,
        now + 1,
        None,
    )
    .expect("valid");
    db::product_variation::update(&pool, &updated)
        .await
        .expect("update");

    let loaded = db::product_variation::get_by_id(&pool, var_id, false)
        .await
        .expect("get_by_id")
        .expect("should exist");
    assert_eq!(loaded.label(), "New label");
    assert_eq!(loaded.unit(), "milliliters");
}

#[tokio::test]
async fn product_variation_update_returns_error_when_not_found() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var =
        ProductVariation::new(var_id, product_id, "X", "none", now, now, None).expect("valid");

    let err = db::product_variation::update(&pool, &var)
        .await
        .expect_err("update should fail");
    assert!(matches!(err, db::DbError::InvalidData(_)));
}

#[tokio::test]
async fn product_variation_update_returns_error_when_already_deleted() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var =
        ProductVariation::new(var_id, product_id, "X", "none", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert");
    db::product_variation::soft_delete(&pool, var_id)
        .await
        .expect("soft_delete");

    let deleted_ts = now + 1;
    let deleted_var = ProductVariation::new(
        var_id,
        product_id,
        "Y",
        "none",
        now,
        deleted_ts,
        Some(deleted_ts),
    )
    .expect("valid");
    let err = db::product_variation::update(&pool, &deleted_var)
        .await
        .expect_err("update deleted should fail");
    assert!(matches!(err, db::DbError::InvalidData(_)));
}

#[tokio::test]
async fn product_variation_soft_delete_then_excluded_from_get_and_list() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var =
        ProductVariation::new(var_id, product_id, "Del", "none", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert");

    db::product_variation::soft_delete(&pool, var_id)
        .await
        .expect("soft_delete");

    let got = db::product_variation::get_by_id(&pool, var_id, false)
        .await
        .expect("get_by_id");
    assert!(got.is_none());

    let list = db::product_variation::list_by_product_id(&pool, product_id, false)
        .await
        .expect("list");
    assert!(list.is_empty());
}

#[tokio::test]
async fn product_variation_soft_delete_returns_error_when_not_found() {
    let (_dir, pool) = setup_pool().await;
    let id = Uuid::new_v4();
    let err = db::product_variation::soft_delete(&pool, id)
        .await
        .expect_err("soft_delete should fail");
    assert!(matches!(err, db::DbError::InvalidData(_)));
}

#[tokio::test]
async fn product_variation_soft_delete_returns_error_when_already_deleted() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var = ProductVariation::new(var_id, product_id, "", "none", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert");
    db::product_variation::soft_delete(&pool, var_id)
        .await
        .expect("first soft_delete");
    let err = db::product_variation::soft_delete(&pool, var_id)
        .await
        .expect_err("second soft_delete should fail");
    assert!(matches!(err, db::DbError::InvalidData(_)));
}

#[tokio::test]
async fn product_variation_count_by_product_id() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let count = db::product_variation::count_by_product_id(&pool, product_id, false)
        .await
        .expect("count");
    assert_eq!(count, 0);

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var = ProductVariation::new(var_id, product_id, "", "none", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert");

    let count = db::product_variation::count_by_product_id(&pool, product_id, false)
        .await
        .expect("count");
    assert_eq!(count, 1);

    db::product_variation::soft_delete(&pool, var_id)
        .await
        .expect("soft_delete");
    let count_active = db::product_variation::count_by_product_id(&pool, product_id, false)
        .await
        .expect("count");
    assert_eq!(count_active, 0);
    let count_all = db::product_variation::count_by_product_id(&pool, product_id, true)
        .await
        .expect("count");
    assert_eq!(count_all, 1);
}

#[tokio::test]
async fn product_variation_ensure_no_purchases_ok_when_no_purchases() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var = ProductVariation::new(var_id, product_id, "", "none", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert");

    db::product_variation::ensure_no_purchases(&pool, var_id)
        .await
        .expect("ensure_no_purchases should succeed");
}

#[tokio::test]
async fn product_variation_ensure_no_purchases_returns_error_when_has_purchases() {
    let (_dir, pool) = setup_pool().await;
    let (_cat_id, product_id) = insert_category_and_product(&pool).await;

    let now = 1_000_i64;
    let var_id = Uuid::new_v4();
    let var = ProductVariation::new(var_id, product_id, "", "none", now, now, None).expect("valid");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert");

    let user_id = Uuid::new_v4();
    let user = User::new(
        user_id,
        "U".to_string(),
        "u@x.com".to_string(),
        "hash".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::user::insert(&pool, &user).await.expect("insert user");

    let loc_id = Uuid::new_v4();
    let loc = Location::new(loc_id, "Store".to_string(), None).expect("valid");
    db::location::insert(&pool, &loc)
        .await
        .expect("insert location");

    let purchase_id = Uuid::new_v4();
    let purchase = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        var_id,
        loc_id,
        1_i32,
        Decimal::new(1, 0),
        now,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert purchase");

    let err = db::product_variation::ensure_no_purchases(&pool, var_id)
        .await
        .expect_err("ensure_no_purchases should fail");
    assert!(matches!(err, db::DbError::InvalidData(_)));
}
