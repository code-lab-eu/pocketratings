//! Integration tests for purchase DB functions.

use pocketratings::db;
use pocketratings::domain::category::Category;
use pocketratings::domain::location::Location;
use pocketratings::domain::product::Product;
use pocketratings::domain::purchase::Purchase;
use pocketratings::domain::user::User;
use rust_decimal::Decimal;
use uuid::Uuid;

async fn insert_user_product_location(pool: &sqlx::SqlitePool) -> (Uuid, Uuid, Uuid) {
    let now = 1_000_i64;

    let user_id = Uuid::new_v4();
    let user = User::new(
        user_id,
        "Test User".to_string(),
        "u@example.com".to_string(),
        "hash".to_string(),
        now,
        now,
        None,
    )
    .expect("valid user");
    db::user::insert(pool, &user).await.expect("insert user");

    let cat_id = Uuid::new_v4();
    let cat = Category::new(cat_id, None, "C".to_string(), now, now, None).expect("valid");
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

    let location_id = Uuid::new_v4();
    let location = Location::new(location_id, "Store".to_string(), None).expect("valid");
    db::location::insert(pool, &location)
        .await
        .expect("insert location");

    (user_id, product_id, location_id)
}

#[tokio::test]
async fn purchase_insert_and_get_by_id_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_insert.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let purchase_id = Uuid::new_v4();
    let price: Decimal = "12.50".parse().expect("decimal");
    let purchase = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        2,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert");

    let loaded = db::purchase::get_by_id(&pool, purchase_id)
        .await
        .expect("get_by_id")
        .expect("purchase should exist");
    assert_eq!(loaded.id(), purchase_id);
    assert_eq!(loaded.quantity(), 2);
    assert_eq!(loaded.price(), price);
}

#[tokio::test]
async fn purchase_list_with_filters_and_include_deleted() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_list.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let price: Decimal = "1".parse().expect("decimal");
    let p1 = Purchase::new(
        Uuid::new_v4(),
        user_id,
        product_id,
        location_id,
        1,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &p1).await.expect("insert");

    let all = db::purchase::list(&pool, None, None, None, None, None, false)
        .await
        .expect("list");
    assert_eq!(all.len(), 1);

    let by_user = db::purchase::list(&pool, Some(user_id), None, None, None, None, false)
        .await
        .expect("list");
    assert_eq!(by_user.len(), 1);

    db::purchase::soft_delete(&pool, p1.id())
        .await
        .expect("soft_delete");

    let active = db::purchase::list(&pool, None, None, None, None, None, false)
        .await
        .expect("list");
    assert_eq!(active.len(), 0);

    let with_deleted = db::purchase::list(&pool, None, None, None, None, None, true)
        .await
        .expect("list");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
async fn purchase_soft_delete_sets_deleted_at_and_excludes_from_get_by_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_soft_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let purchase_id = Uuid::new_v4();
    let price: Decimal = "5".parse().expect("decimal");
    let purchase = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        1,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert");

    db::purchase::soft_delete(&pool, purchase_id)
        .await
        .expect("soft_delete");

    let by_id = db::purchase::get_by_id(&pool, purchase_id)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());

    let with_deleted = db::purchase::list(&pool, None, None, None, None, None, true)
        .await
        .expect("list");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
async fn purchase_hard_delete_removes_row() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_hard_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let purchase_id = Uuid::new_v4();
    let price: Decimal = "3.99".parse().expect("decimal");
    let purchase = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        1,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert");

    db::purchase::hard_delete(&pool, purchase_id)
        .await
        .expect("hard_delete");

    let by_id = db::purchase::get_by_id(&pool, purchase_id)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());

    let with_deleted = db::purchase::list(&pool, None, None, None, None, None, true)
        .await
        .expect("list");
    assert!(with_deleted.is_empty());
}

#[tokio::test]
async fn purchase_update_changes_quantity_and_price() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let purchase_id = Uuid::new_v4();
    let price: Decimal = "2.00".parse().expect("decimal");
    let purchase = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        1,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert");

    // Update quantity and price.
    let updated_price: Decimal = "3.50".parse().expect("decimal");
    let updated = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        3,
        updated_price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::update(&pool, &updated).await.expect("update");

    let loaded = db::purchase::get_by_id(&pool, purchase_id)
        .await
        .expect("get_by_id")
        .expect("purchase should exist");
    assert_eq!(loaded.quantity(), 3);
    assert_eq!(loaded.price(), updated_price);
}
