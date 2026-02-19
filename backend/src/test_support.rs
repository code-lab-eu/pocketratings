//! Shared test helpers for backend unit and integration tests.
//!
//! This module is only compiled when running tests.

#![allow(clippy::missing_panics_doc)]

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db;
use crate::domain::category::Category;
use crate::domain::location::Location;
use crate::domain::product::Product;

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

/// Insert a test location and return its id.
pub async fn insert_location(pool: &SqlitePool, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    let loc = Location::new(id, name.to_string(), None).expect("valid location");
    db::location::insert(pool, &loc)
        .await
        .expect("insert location");
    id
}
