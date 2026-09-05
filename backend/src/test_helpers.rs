//! Shared test helpers for backend unit and integration tests.
//!
//! This module is only compiled when running tests.

#![allow(clippy::missing_panics_doc)]

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::AppState;
use crate::config::Config;
use crate::db;
use crate::domain::category::Category;
use crate::domain::location::Location;
use crate::domain::product::Product;
use crate::domain::product_variation::ProductVariation;

/// Build an [`AppState`] backed by a migrated temp-file database, for endpoint tests.
///
/// The returned [`tempfile::TempDir`] must be kept alive for the duration of the test, otherwise
/// the database file is removed.
pub async fn api_test_state() -> (AppState, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("api_test.db");
    let path_str = db_path.to_str().expect("path utf-8").to_string();
    let pool = db::create_pool(&path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrate");
    let state = AppState {
        config: Config::for_tests(&path_str),
        pool,
    };
    (state, dir)
}

/// Insert a test user into the database and return its id.
pub async fn insert_user(pool: &SqlitePool, name: &str, email: &str) -> Uuid {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp();
    let hash = crate::auth::password::hash_password("pass").expect("hash");
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(name)
    .bind(email)
    .bind(&hash)
    .bind(now)
    .bind(now)
    .bind::<Option<i64>>(None)
    .execute(pool)
    .await
    .expect("insert user");
    id
}

/// Insert a test category and return its id.
pub async fn insert_category(pool: &SqlitePool, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp();
    let cat = Category::new(id, None, name.to_string(), now, now, None).expect("valid category");
    db::category::insert(pool, &cat)
        .await
        .expect("insert category");
    id
}

/// Insert a test product and return its id.
pub async fn insert_product(pool: &SqlitePool, category_id: Uuid, brand: &str, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp();
    let product = Product::new(
        id,
        category_id,
        brand.to_string(),
        name.to_string(),
        now,
        now,
        None,
    )
    .expect("valid product");
    db::product::insert(pool, &product)
        .await
        .expect("insert product");
    id
}

/// Ensure the product has at least one variation; insert a default one if not. Returns variation id.
pub async fn ensure_product_variation(pool: &SqlitePool, product_id: Uuid) -> Uuid {
    let existing = db::product_variation::list_by_product_id(pool, product_id, false)
        .await
        .expect("list_by_product_id");
    if let Some(v) = existing.first() {
        return v.id();
    }
    let now = chrono::Utc::now().timestamp();
    let var_id = Uuid::new_v4();
    let var = ProductVariation::new(var_id, product_id, "", "none", None, now, now, None)
        .expect("valid variation");
    db::product_variation::insert(pool, &var)
        .await
        .expect("insert variation");
    var_id
}

/// Insert a test location and return its id.
pub async fn insert_location(pool: &SqlitePool, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    let loc = Location::new(id, name.to_string(), None).expect("valid location");
    db::location::insert(pool, &loc)
        .await
        .expect("insert location");
    id
}
