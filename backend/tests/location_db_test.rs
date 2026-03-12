//! Integration tests for location DB functions.

use pocketratings::db;
use pocketratings::domain::location::Location;
use pocketratings::domain::product_variation::ProductVariation;
use pocketratings::domain::purchase::Purchase;
use rust_decimal::Decimal;
use serial_test::serial;
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

    let loaded = db::location::get_by_id(&pool, loc_id, false)
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

    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 2);

    db::location::soft_delete(&pool, l1.id())
        .await
        .expect("soft_delete");

    let active = db::location::get_all(&pool, false).await.expect("get_all");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].name(), "Store B");

    let with_deleted = db::location::get_all(&pool, true)
        .await
        .expect("get_all(include_deleted: true)");
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

    let loaded = db::location::get_by_id(&pool, loc_id, false)
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

    let by_id = db::location::get_by_id(&pool, loc_id, false)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());

    let by_id_incl = db::location::get_by_id(&pool, loc_id, true)
        .await
        .expect("get_by_id");
    assert!(
        by_id_incl.is_some(),
        "get_by_id(include_deleted: true) must return soft-deleted location"
    );
    assert_eq!(by_id_incl.as_ref().unwrap().name(), "ToDelete");

    let with_deleted = db::location::get_all(&pool, true)
        .await
        .expect("get_all(include_deleted: true)");
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

    let by_id = db::location::get_by_id(&pool, loc_id, false)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());
    let with_deleted = db::location::get_all(&pool, true)
        .await
        .expect("get_all(include_deleted: true)");
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

    let var_id = Uuid::new_v4();
    let var = ProductVariation::new(var_id, product_id, "", "none", now, now, None)
        .expect("valid variation");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert variation");

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

    let purchase = Purchase::new(
        Uuid::new_v4(),
        user_id,
        product_id,
        var_id,
        loc_id,
        1,
        "9.99".parse::<Decimal>().expect("decimal"),
        now,
        None,
    )
    .expect("valid purchase");
    db::purchase::insert(&pool, &purchase)
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

    let var_id = Uuid::new_v4();
    let var = ProductVariation::new(var_id, product_id, "", "none", now, now, None)
        .expect("valid variation");
    db::product_variation::insert(&pool, &var)
        .await
        .expect("insert variation");

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

    let purchase = Purchase::new(
        Uuid::new_v4(),
        user_id,
        product_id,
        var_id,
        loc_id,
        1,
        "9.99".parse::<Decimal>().expect("decimal"),
        now,
        None,
    )
    .expect("valid purchase");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert purchase");

    let result = db::location::hard_delete(&pool, loc_id).await;
    assert!(
        result.is_err(),
        "hard_delete should fail when location has purchases"
    );
}

// --- Location list cache tests (run serially; they enable the cache for the process) ---

#[tokio::test]
#[serial]
async fn location_list_cache_is_warmed_on_first_call_and_used_on_subsequent_calls() {
    db::location::clear_location_list_cache();
    db::location::set_use_location_list_cache_for_test(true);
    let _guard = LocationCacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_cache_warm.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert!(all.is_empty(), "first call: empty DB -> empty list");

    let cached = Location::new(Uuid::new_v4(), "CachedOnly".to_string(), None).expect("valid");
    db::location::set_location_list_cache_for_test(Some(vec![cached.clone()]));

    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1, "must return cached list, not DB");
    assert_eq!(all[0].name(), "CachedOnly");

    let with_del = db::location::get_all(&pool, true)
        .await
        .expect("get_all with include_deleted");
    assert_eq!(with_del.len(), 1);
    assert_eq!(with_del[0].name(), "CachedOnly");
}

#[tokio::test]
#[serial]
async fn location_list_cache_invalidated_after_insert() {
    db::location::clear_location_list_cache();
    db::location::set_use_location_list_cache_for_test(true);
    let _guard = LocationCacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_cache_insert.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert!(all.is_empty());
    db::location::set_location_list_cache_for_test(Some(vec![]));
    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert!(all.is_empty(), "cached empty");

    let loc = Location::new(Uuid::new_v4(), "New".to_string(), None).expect("valid");
    db::location::insert(&pool, &loc).await.expect("insert");

    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1, "cache must be invalidated after insert");
    assert_eq!(all[0].name(), "New");
}

#[tokio::test]
#[serial]
async fn location_list_cache_invalidated_after_update() {
    db::location::clear_location_list_cache();
    db::location::set_use_location_list_cache_for_test(true);
    let _guard = LocationCacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_cache_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    let loc = Location::new(id, "Original".to_string(), None).expect("valid");
    db::location::insert(&pool, &loc).await.expect("insert");
    let _ = db::location::get_all(&pool, false).await.expect("get_all");

    let stale = Location::new(Uuid::new_v4(), "Stale".to_string(), None).expect("valid");
    db::location::set_location_list_cache_for_test(Some(vec![stale]));

    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name(), "Stale", "still cached");

    let updated = Location::new(id, "Updated".to_string(), None).expect("valid");
    db::location::update(&pool, &updated).await.expect("update");

    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1, "cache must be invalidated after update");
    assert_eq!(all[0].name(), "Updated");
}

#[tokio::test]
#[serial]
async fn location_list_cache_invalidated_after_soft_delete() {
    db::location::clear_location_list_cache();
    db::location::set_use_location_list_cache_for_test(true);
    let _guard = LocationCacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_cache_soft_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    let loc = Location::new(id, "ToDelete".to_string(), None).expect("valid");
    db::location::insert(&pool, &loc).await.expect("insert");
    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1);

    db::location::soft_delete(&pool, id)
        .await
        .expect("soft_delete");

    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert!(
        all.is_empty(),
        "cache must be invalidated; get_all shows active only"
    );
    let with_del = db::location::get_all(&pool, true)
        .await
        .expect("get_all with include_deleted");
    assert_eq!(with_del.len(), 1);
}

#[tokio::test]
#[serial]
async fn location_list_cache_invalidated_after_hard_delete() {
    db::location::clear_location_list_cache();
    db::location::set_use_location_list_cache_for_test(true);
    let _guard = LocationCacheTestGuard;

    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("location_cache_hard_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let id = Uuid::new_v4();
    let loc = Location::new(id, "ToRemove".to_string(), None).expect("valid");
    db::location::insert(&pool, &loc).await.expect("insert");
    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert_eq!(all.len(), 1);

    db::location::hard_delete(&pool, id)
        .await
        .expect("hard_delete");

    let all = db::location::get_all(&pool, false).await.expect("get_all");
    assert!(
        all.is_empty(),
        "cache must be invalidated after hard_delete"
    );
}

struct LocationCacheTestGuard;

impl Drop for LocationCacheTestGuard {
    fn drop(&mut self) {
        db::location::clear_location_list_cache();
        db::location::set_use_location_list_cache_for_test(false);
    }
}
