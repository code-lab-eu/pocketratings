//! Integration tests for review DB functions.

use pocketratings::db;
use pocketratings::db::review::ReviewWithRelations;
use pocketratings::domain::category::Category;
use pocketratings::domain::product::Product;
use pocketratings::domain::review::Review;
use pocketratings::domain::user::User;
use rust_decimal::Decimal;
use serial_test::serial;
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
async fn review_get_by_id_with_relations_returns_none_for_nonexistent() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_get_rel.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let got = db::review::get_by_id_with_relations(&pool, Uuid::new_v4())
        .await
        .expect("get_by_id_with_relations");
    assert!(got.is_none());
}

#[tokio::test]
async fn review_get_by_id_with_relations_returns_review_and_relation_names() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_get_rel_names.db");
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

    let with_rel = db::review::get_by_id_with_relations(&pool, review_id)
        .await
        .expect("get_by_id_with_relations")
        .expect("review should exist");
    assert_eq!(with_rel.id, review_id);
    assert_eq!(with_rel.user_name, "Test User");
    assert_eq!(with_rel.product_brand, "Brand");
    assert_eq!(with_rel.product_name, "Product");
    assert_eq!(with_rel.rating, "4");
    assert_eq!(with_rel.text.as_deref(), Some("Good"));
}

#[tokio::test]
async fn review_list_with_relations_returns_relation_names_and_filters() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_list_rel.db");
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

    let list = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].user_name, "Test User");
    assert_eq!(list[0].product_brand, "Brand");
    assert_eq!(list[0].product_name, "Product");

    let by_product = db::review::list_with_relations(&pool, Some(product_id), None, false)
        .await
        .expect("list_with_relations");
    assert_eq!(by_product.len(), 1);

    let by_user = db::review::list_with_relations(&pool, None, Some(user_id), false)
        .await
        .expect("list_with_relations");
    assert_eq!(by_user.len(), 1);

    db::review::soft_delete(&pool, r1.id())
        .await
        .expect("soft_delete");
    let active = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert_eq!(active.len(), 0);
    let with_deleted = db::review::list_with_relations(&pool, None, None, true)
        .await
        .expect("list_with_relations");
    assert_eq!(with_deleted.len(), 1);
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

// --- Review list cache tests (run serially; they enable the cache for the process) ---

#[tokio::test]
#[serial]
async fn review_list_cache_is_warmed_on_first_call_and_used_on_subsequent_calls() {
    db::review::clear_review_list_cache();
    db::review::set_use_review_list_cache_for_test(true);
    let _guard = ReviewCacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_cache_warm.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let all = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert!(all.is_empty(), "first call: empty DB -> empty list");

    let (user_id, product_id) = insert_user_and_product(&pool).await;
    let review_id = Uuid::new_v4();
    let cached = ReviewWithRelations {
        id: review_id,
        product_id,
        user_id,
        rating: "4".to_string(),
        text: None,
        created_at: 1,
        updated_at: 1,
        deleted_at: None,
        user_name: "CachedUser".to_string(),
        product_brand: "CachedBrand".to_string(),
        product_name: "CachedProduct".to_string(),
    };
    db::review::set_review_list_cache_for_test(Some(vec![cached.clone()]));

    let all = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert_eq!(all.len(), 1, "must return cached list, not DB");
    assert_eq!(all[0].user_name, "CachedUser");
    assert_eq!(all[0].product_name, "CachedProduct");
}

#[tokio::test]
#[serial]
async fn review_list_cache_invalidated_after_insert() {
    db::review::clear_review_list_cache();
    db::review::set_use_review_list_cache_for_test(true);
    let _guard = ReviewCacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_cache_insert.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id) = insert_user_and_product(&pool).await;
    let _ = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    db::review::set_review_list_cache_for_test(Some(vec![]));
    let all = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert!(all.is_empty(), "cached empty");

    let review_id = Uuid::new_v4();
    let r = Review::new(
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
    db::review::insert(&pool, &r).await.expect("insert");

    let all = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert_eq!(all.len(), 1, "cache must be invalidated after insert");
    assert_eq!(all[0].user_name, "Test User");
}

#[tokio::test]
#[serial]
async fn review_list_cache_invalidated_after_update() {
    db::review::clear_review_list_cache();
    db::review::set_use_review_list_cache_for_test(true);
    let _guard = ReviewCacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_cache_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id) = insert_user_and_product(&pool).await;
    let review_id = Uuid::new_v4();
    let r = Review::new(
        review_id,
        product_id,
        user_id,
        Decimal::from(3),
        Some("Original".to_string()),
        1_000,
        1_000,
        None,
    )
    .expect("valid");
    db::review::insert(&pool, &r).await.expect("insert");
    let _ = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");

    let stale = ReviewWithRelations {
        id: review_id,
        product_id,
        user_id,
        rating: "3".to_string(),
        text: Some("Stale".to_string()),
        created_at: 1_000,
        updated_at: 1_000,
        deleted_at: None,
        user_name: "Test User".to_string(),
        product_brand: "Brand".to_string(),
        product_name: "Product".to_string(),
    };
    db::review::set_review_list_cache_for_test(Some(vec![stale]));

    let all = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].text.as_deref(), Some("Stale"), "still cached");

    let updated = Review::new(
        review_id,
        product_id,
        user_id,
        Decimal::from(5),
        Some("Updated".to_string()),
        1_000,
        2_000,
        None,
    )
    .expect("valid");
    db::review::update(&pool, &updated).await.expect("update");

    let all = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert_eq!(all.len(), 1, "cache must be invalidated after update");
    assert_eq!(all[0].rating, "5");
    assert_eq!(all[0].text.as_deref(), Some("Updated"));
}

#[tokio::test]
#[serial]
async fn review_list_cache_invalidated_after_soft_delete() {
    db::review::clear_review_list_cache();
    db::review::set_use_review_list_cache_for_test(true);
    let _guard = ReviewCacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_cache_soft_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id) = insert_user_and_product(&pool).await;
    let review_id = Uuid::new_v4();
    let r = Review::new(
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
    db::review::insert(&pool, &r).await.expect("insert");
    let all = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert_eq!(all.len(), 1);

    db::review::soft_delete(&pool, review_id)
        .await
        .expect("soft_delete");

    let all = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert!(
        all.is_empty(),
        "cache must be invalidated; list shows active only"
    );
    let with_del = db::review::list_with_relations(&pool, None, None, true)
        .await
        .expect("list_with_relations");
    assert_eq!(with_del.len(), 1);
}

#[tokio::test]
#[serial]
async fn review_list_cache_invalidated_after_hard_delete() {
    db::review::clear_review_list_cache();
    db::review::set_use_review_list_cache_for_test(true);
    let _guard = ReviewCacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("review_cache_hard_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id) = insert_user_and_product(&pool).await;
    let review_id = Uuid::new_v4();
    let r = Review::new(
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
    db::review::insert(&pool, &r).await.expect("insert");
    let all = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert_eq!(all.len(), 1);

    db::review::hard_delete(&pool, review_id)
        .await
        .expect("hard_delete");

    let all = db::review::list_with_relations(&pool, None, None, false)
        .await
        .expect("list_with_relations");
    assert!(
        all.is_empty(),
        "cache must be invalidated after hard_delete"
    );
}

struct ReviewCacheTestGuard;

impl Drop for ReviewCacheTestGuard {
    fn drop(&mut self) {
        db::review::clear_review_list_cache();
        db::review::set_use_review_list_cache_for_test(false);
    }
}
