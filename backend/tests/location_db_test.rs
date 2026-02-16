//! Integration tests for location DB functions.

use pocketratings::db;
use pocketratings::domain::location::Location;
use uuid::Uuid;

#[tokio::test]
async fn location_insert_and_get_by_id_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_insert.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let loc_id = Uuid::new_v4();
    let location = Location::new(loc_id, "Supermarket".to_string(), None).expect("valid");
    db::location::insert(&pool, &location)
        .await
        .expect("insert");

    let loaded = db::location::get_by_id(&pool, loc_id)
        .await
        .expect("get_by_id")
        .expect("location should exist");
    assert_eq!(loaded.id(), loc_id);
    assert_eq!(loaded.name(), "Supermarket");
}

#[tokio::test]
async fn location_get_all_and_get_all_with_deleted() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_get_all.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let l1 = Location::new(Uuid::new_v4(), "Store A".to_string(), None).expect("valid");
    let l2 = Location::new(Uuid::new_v4(), "Store B".to_string(), None).expect("valid");
    db::location::insert(&pool, &l1).await.expect("insert");
    db::location::insert(&pool, &l2).await.expect("insert");

    let all = db::location::get_all(&pool).await.expect("get_all");
    assert_eq!(all.len(), 2);

    db::location::soft_delete(&pool, l1.id())
        .await
        .expect("soft_delete");

    let active = db::location::get_all(&pool).await.expect("get_all");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].name(), "Store B");

    let with_deleted = db::location::get_all_with_deleted(&pool)
        .await
        .expect("get_all_with_deleted");
    assert_eq!(with_deleted.len(), 2);
    assert!(
        with_deleted
            .iter()
            .any(|l| l.name() == "Store A" && !l.is_active())
    );
}

#[tokio::test]
async fn location_update_changes_name() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let loc_id = Uuid::new_v4();
    let location = Location::new(loc_id, "OldName".to_string(), None).expect("valid");
    db::location::insert(&pool, &location)
        .await
        .expect("insert");

    let updated = Location::new(loc_id, "NewName".to_string(), None).expect("valid");
    db::location::update(&pool, &updated).await.expect("update");

    let loaded = db::location::get_by_id(&pool, loc_id)
        .await
        .expect("get_by_id")
        .expect("should exist");
    assert_eq!(loaded.name(), "NewName");
}

#[tokio::test]
async fn location_soft_delete_sets_deleted_at_and_excludes_from_get_by_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_soft_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let loc_id = Uuid::new_v4();
    let location = Location::new(loc_id, "ToDelete".to_string(), None).expect("valid");
    db::location::insert(&pool, &location)
        .await
        .expect("insert");

    db::location::soft_delete(&pool, loc_id)
        .await
        .expect("soft_delete");

    let by_id = db::location::get_by_id(&pool, loc_id)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());

    let with_deleted = db::location::get_all_with_deleted(&pool)
        .await
        .expect("get_all_with_deleted");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
async fn location_hard_delete_removes_row() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_hard_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let loc_id = Uuid::new_v4();
    let location = Location::new(loc_id, "ToRemove".to_string(), None).expect("valid");
    db::location::insert(&pool, &location)
        .await
        .expect("insert");

    db::location::hard_delete(&pool, loc_id)
        .await
        .expect("hard_delete");

    let by_id = db::location::get_by_id(&pool, loc_id)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());
    let with_deleted = db::location::get_all_with_deleted(&pool)
        .await
        .expect("get_all_with_deleted");
    assert!(with_deleted.is_empty());
}

#[tokio::test]
async fn location_soft_delete_fails_when_location_has_purchases() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_soft_delete_purchases.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let loc_id = Uuid::new_v4();
    let location = Location::new(loc_id, "Store".to_string(), None).expect("valid");
    db::location::insert(&pool, &location)
        .await
        .expect("insert location");

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
    let product = pocketratings::domain::product::Product::new(
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

    sqlx::query(
        "INSERT INTO purchases (id, user_id, product_id, location_id, quantity, price, purchased_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id.to_string())
    .bind(product_id.to_string())
    .bind(loc_id.to_string())
    .bind(1_i32)
    .bind("9.99")
    .bind(now)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert purchase");

    let result = db::location::soft_delete(&pool, loc_id).await;
    assert!(
        result.is_err(),
        "soft_delete should fail when location has purchases"
    );
}

#[tokio::test]
async fn location_hard_delete_fails_when_location_has_purchases() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_hard_delete_purchases.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let loc_id = Uuid::new_v4();
    let location = Location::new(loc_id, "Store".to_string(), None).expect("valid");
    db::location::insert(&pool, &location)
        .await
        .expect("insert location");

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
    let product = pocketratings::domain::product::Product::new(
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

    sqlx::query(
        "INSERT INTO purchases (id, user_id, product_id, location_id, quantity, price, purchased_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id.to_string())
    .bind(product_id.to_string())
    .bind(loc_id.to_string())
    .bind(1_i32)
    .bind("9.99")
    .bind(now)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert purchase");

    let result = db::location::hard_delete(&pool, loc_id).await;
    assert!(
        result.is_err(),
        "hard_delete should fail when location has purchases"
    );
}
