//! Integration tests for review DB functions.

use pocketratings::db;
use pocketratings::domain::category::Category;
use pocketratings::domain::product::Product;
use pocketratings::domain::review::Review;
use pocketratings::domain::user::User;
use rust_decimal::Decimal;
use uuid::Uuid;

async fn insert_user_and_product(pool: &sqlx::SqlitePool) -> (Uuid, Uuid) {
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

    (user_id, product_id)
}

#[tokio::test]
async fn review_insert_and_get_by_id_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_insert.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id) = insert_user_and_product(&pool).await;

    let review_id = Uuid::new_v4();
    let review = Review::new(
        review_id,
        product_id,
        user_id,
        Decimal::from(4),
        Some("Good".to_string()),
        1_000,
        1_000,
        None,
    )
    .expect("valid");
    db::review::insert(&pool, &review).await.expect("insert");

    let loaded = db::review::get_by_id(&pool, review_id)
        .await
        .expect("get_by_id")
        .expect("review should exist");
    assert_eq!(loaded.id(), review_id);
    assert_eq!(loaded.rating(), Decimal::from(4));
    assert_eq!(loaded.text(), Some("Good"));
}

#[tokio::test]
async fn review_list_with_filters_and_include_deleted() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_list.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id) = insert_user_and_product(&pool).await;

    let r1 = Review::new(
        Uuid::new_v4(),
        product_id,
        user_id,
        Decimal::from(3),
        None,
        1_000,
        1_000,
        None,
    )
    .expect("valid");
    db::review::insert(&pool, &r1).await.expect("insert");

    let all = db::review::list(&pool, None, None, false)
        .await
        .expect("list");
    assert_eq!(all.len(), 1);

    let by_product = db::review::list(&pool, Some(product_id), None, false)
        .await
        .expect("list");
    assert_eq!(by_product.len(), 1);

    let by_user = db::review::list(&pool, None, Some(user_id), false)
        .await
        .expect("list");
    assert_eq!(by_user.len(), 1);

    db::review::soft_delete(&pool, r1.id())
        .await
        .expect("soft_delete");

    let active = db::review::list(&pool, None, None, false)
        .await
        .expect("list");
    assert_eq!(active.len(), 0);

    let with_deleted = db::review::list(&pool, None, None, true)
        .await
        .expect("list");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
async fn review_update_changes_rating_and_text() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id) = insert_user_and_product(&pool).await;

    let review_id = Uuid::new_v4();
    let review = Review::new(
        review_id,
        product_id,
        user_id,
        Decimal::from(3),
        Some("Ok".to_string()),
        1_000,
        1_000,
        None,
    )
    .expect("valid");
    db::review::insert(&pool, &review).await.expect("insert");

    let updated = Review::new(
        review_id,
        product_id,
        user_id,
        Decimal::from(5),
        Some("Great!".to_string()),
        1_000,
        2_000,
        None,
    )
    .expect("valid");
    db::review::update(&pool, &updated).await.expect("update");

    let loaded = db::review::get_by_id(&pool, review_id)
        .await
        .expect("get_by_id")
        .expect("should exist");
    assert_eq!(loaded.rating(), Decimal::from(5));
    assert_eq!(loaded.text(), Some("Great!"));
    assert_eq!(loaded.updated_at(), 2_000);
}

#[tokio::test]
async fn review_soft_delete_sets_deleted_at_and_excludes_from_get_by_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_soft_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id) = insert_user_and_product(&pool).await;

    let review_id = Uuid::new_v4();
    let review = Review::new(
        review_id,
        product_id,
        user_id,
        Decimal::from(4),
        None,
        1_000,
        1_000,
        None,
    )
    .expect("valid");
    db::review::insert(&pool, &review).await.expect("insert");

    db::review::soft_delete(&pool, review_id)
        .await
        .expect("soft_delete");

    let by_id = db::review::get_by_id(&pool, review_id)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());

    let with_deleted = db::review::list(&pool, None, None, true)
        .await
        .expect("list");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
async fn review_hard_delete_removes_row() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_hard_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id) = insert_user_and_product(&pool).await;

    let review_id = Uuid::new_v4();
    let review = Review::new(
        review_id,
        product_id,
        user_id,
        Decimal::from(4),
        None,
        1_000,
        1_000,
        None,
    )
    .expect("valid");
    db::review::insert(&pool, &review).await.expect("insert");

    db::review::hard_delete(&pool, review_id)
        .await
        .expect("hard_delete");

    let by_id = db::review::get_by_id(&pool, review_id)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());

    let with_deleted = db::review::list(&pool, None, None, true)
        .await
        .expect("list");
    assert!(with_deleted.is_empty());
}
