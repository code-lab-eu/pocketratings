//! Products REST API: list, get, create, update, delete.

use axum::routing::get;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::category::CategoryRef;
use crate::api::{error::ApiError, state::AppState};
use crate::db;
use crate::domain::product::Product;

/// Minimal product info for embedding in purchase (and future) responses.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProductRef {
    pub id: Uuid,
    pub brand: String,
    pub name: String,
}

/// Request body for creating a product.
#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub category_id: Uuid,
    pub brand: String,
    pub name: String,
}

/// Request body for partial update.
#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub category_id: Option<Uuid>,
    pub brand: Option<String>,
    pub name: Option<String>,
}

/// Query params for list products.
#[derive(Debug, Default, Deserialize)]
pub struct ListProductsQuery {
    pub category_id: Option<Uuid>,
    pub q: Option<String>,
}

/// Query params for delete (optional force).
#[derive(Debug, Default, Deserialize)]
pub struct DeleteProductQuery {
    #[serde(default, deserialize_with = "parse_force")]
    pub force: bool,
}

/// Response body: product with timestamps as i64 and nested category.
#[derive(Debug, serde::Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub category: CategoryRef,
    pub brand: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

fn product_with_relations_to_response(p: &db::product::ProductWithRelations) -> ProductResponse {
    ProductResponse {
        id: p.id,
        category: CategoryRef {
            id: p.category_id,
            name: p.category_name.clone(),
        },
        brand: p.brand.clone(),
        name: p.name.clone(),
        created_at: p.created_at,
        updated_at: p.updated_at,
        deleted_at: p.deleted_at,
    }
}

/// Map `DbError` to `ApiError` for product operations.
fn map_db_error(e: &db::DbError) -> ApiError {
    match e {
        db::DbError::InvalidData(msg) => {
            if msg.contains("cannot delete") {
                ApiError::Conflict(msg.clone())
            } else {
                ApiError::BadRequest(msg.clone())
            }
        }
        db::DbError::Sqlx(sqlx_err) => {
            if let sqlx::Error::Database(db) = sqlx_err
                && (db.is_unique_violation() || db.is_foreign_key_violation())
            {
                return ApiError::BadRequest(e.to_string());
            }
            ApiError::Internal
        }
        db::DbError::Migrate(_) => ApiError::Internal,
    }
}

/// Deserialize "true" / "false" for force query param.
pub fn parse_force<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
    Ok(s.to_lowercase() == "true" || s == "1")
}

/// GET /api/v1/products — list products, optionally filtered by `category_id` and/or `q` (search).
pub async fn list_products(
    State(state): State<AppState>,
    Query(q): Query<ListProductsQuery>,
) -> Result<Json<Vec<ProductResponse>>, ApiError> {
    let list = db::product::list_with_relations(&state.pool, q.category_id, q.q.as_deref(), false)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok(Json(
        list.iter()
            .map(product_with_relations_to_response)
            .collect(),
    ))
}

/// GET /api/v1/products/:id — get one product.
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductResponse>, ApiError> {
    let product = db::product::get_by_id_with_relations(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let product = product.ok_or_else(|| ApiError::NotFound("product not found".to_string()))?;
    Ok(Json(product_with_relations_to_response(&product)))
}

/// POST /api/v1/products — create a product.
pub async fn create_product(
    State(state): State<AppState>,
    Json(body): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<ProductResponse>), ApiError> {
    let category = db::category::get_by_id(&state.pool, body.category_id)
        .await
        .map_err(|e| map_db_error(&e))?;
    if category.is_none() {
        return Err(ApiError::NotFound("category not found".to_string()));
    }

    if body.brand.trim().is_empty() {
        return Err(ApiError::BadRequest("brand must not be empty".to_string()));
    }
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name must not be empty".to_string()));
    }

    let now = chrono::Utc::now().timestamp();
    let id = Uuid::new_v4();
    let product = Product::new(
        id,
        body.category_id,
        body.brand.trim().to_string(),
        body.name.trim().to_string(),
        now,
        now,
        None,
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    db::product::insert(&state.pool, &product)
        .await
        .map_err(|e| map_db_error(&e))?;
    let created = db::product::get_by_id_with_relations(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?
        .expect("product just inserted");
    Ok((
        StatusCode::CREATED,
        Json(product_with_relations_to_response(&created)),
    ))
}

/// PATCH /api/v1/products/:id — partial update; only persist if something changed.
pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, ApiError> {
    let existing = db::product::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let existing = existing.ok_or_else(|| ApiError::NotFound("product not found".to_string()))?;

    let new_category_id = body.category_id.or_else(|| Some(existing.category_id()));
    let category_id = new_category_id.unwrap();
    if body.category_id.is_some() {
        let category = db::category::get_by_id(&state.pool, category_id)
            .await
            .map_err(|e| map_db_error(&e))?;
        if category.is_none() {
            return Err(ApiError::NotFound("category not found".to_string()));
        }
    }

    let brand = body
        .brand
        .as_deref()
        .map_or_else(|| existing.brand().to_string(), |s| s.trim().to_string());
    let name = body
        .name
        .as_deref()
        .map_or_else(|| existing.name().to_string(), |s| s.trim().to_string());

    if brand.trim().is_empty() {
        return Err(ApiError::BadRequest("brand must not be empty".to_string()));
    }
    if name.trim().is_empty() {
        return Err(ApiError::BadRequest("name must not be empty".to_string()));
    }

    if existing.category_id() == category_id && existing.brand() == brand && existing.name() == name
    {
        let current = db::product::get_by_id_with_relations(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?
            .expect("product exists");
        return Ok(Json(product_with_relations_to_response(&current)));
    }

    let updated = Product::new(
        existing.id(),
        category_id,
        brand,
        name,
        existing.created_at(),
        chrono::Utc::now().timestamp(),
        existing.deleted_at(),
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    db::product::update(&state.pool, &updated)
        .await
        .map_err(|e| map_db_error(&e))?;
    let current = db::product::get_by_id_with_relations(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?
        .expect("product exists");
    Ok(Json(product_with_relations_to_response(&current)))
}

/// DELETE /api/v1/products/:id — soft delete, or hard with ?force=true.
pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<DeleteProductQuery>,
) -> Result<StatusCode, ApiError> {
    let _ = db::product::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?
        .ok_or_else(|| ApiError::NotFound("product not found".to_string()))?;

    if q.force {
        db::product::hard_delete(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?;
    } else {
        db::product::soft_delete(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Router for /api/v1/products (all five handlers).
pub fn route() -> Router<AppState> {
    Router::new()
        .route("/api/v1/products", get(list_products).post(create_product))
        .route(
            "/api/v1/products/{id}",
            get(get_product)
                .patch(update_product)
                .delete(delete_product),
        )
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use sqlx::SqlitePool;

    use super::*;
    use crate::config::Config;
    use crate::db;
    use crate::domain::category::Category;

    async fn test_pool() -> (AppState, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("product_test.db");
        let path_str = db_path.to_str().expect("path utf-8").to_string();
        let pool = db::create_pool(&path_str).await.expect("pool");
        db::run_migrations(&pool).await.expect("migrate");
        let state = AppState {
            config: Config {
                database_path: path_str,
                jwt_secret: "test".to_string(),
                jwt_expiration_seconds: 3600,
                jwt_refresh_threshold_seconds: 600,
                bind: "127.0.0.1:0".to_string(),
                pid_file: std::env::temp_dir()
                    .join("pocketratings-product-test.pid")
                    .to_string_lossy()
                    .into_owned(),
            },
            pool,
        };
        (state, dir)
    }

    async fn insert_category(pool: &SqlitePool, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp();
        let cat =
            Category::new(id, None, name.to_string(), now, now, None).expect("valid category");
        db::category::insert(pool, &cat)
            .await
            .expect("insert category");
        id
    }

    #[tokio::test]
    async fn list_products_returns_empty_array_when_none() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/products")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert!(json.is_array());
        assert!(json.as_array().expect("array").is_empty());
    }

    #[tokio::test]
    async fn create_product_returns_201_and_body() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Groceries").await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "Acme",
            "name": "Widget"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::CREATED);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(json.get("brand").and_then(|v| v.as_str()), Some("Acme"));
        assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("Widget"));
        let category = json.get("category").expect("category");
        assert_eq!(
            category.get("id").and_then(|v| v.as_str()),
            Some(cat_id.to_string().as_str())
        );
        assert_eq!(
            category.get("name").and_then(|v| v.as_str()),
            Some("Groceries")
        );
        let id_str = json.get("id").and_then(|v| v.as_str()).expect("id");
        assert!(Uuid::parse_str(id_str).is_ok());
        assert!(
            json.get("created_at")
                .and_then(serde_json::Value::as_i64)
                .is_some()
        );
        assert!(
            json.get("updated_at")
                .and_then(serde_json::Value::as_i64)
                .is_some()
        );
        assert!(
            json.get("deleted_at")
                .is_none_or(serde_json::Value::is_null)
        );
        let id = Uuid::parse_str(id_str).expect("uuid");
        let persisted = db::product::get_by_id(&state.pool, id).await.expect("db");
        let persisted = persisted.expect("product in db");
        assert_eq!(persisted.brand(), "Acme");
        assert_eq!(persisted.name(), "Widget");
        assert_eq!(persisted.category_id(), cat_id);
    }

    #[tokio::test]
    async fn create_product_returns_404_when_category_not_found() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let fake_cat_id = Uuid::new_v4();
        let body = serde_json::json!({
            "category_id": fake_cat_id.to_string(),
            "brand": "B",
            "name": "N"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn create_product_returns_400_when_brand_empty() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state);
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "",
            "name": "N"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_product_returns_400_when_name_empty() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state);
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "B",
            "name": ""
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_product_returns_404_when_not_found() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/products/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_product_returns_200_when_found() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state);
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "B",
            "name": "P"
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let bytes = create_resp
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let created: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let id = created.get("id").and_then(|v| v.as_str()).expect("id");
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/products/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(json.get("id").and_then(|v| v.as_str()), Some(id));
        assert_eq!(json.get("brand").and_then(|v| v.as_str()), Some("B"));
        assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("P"));
    }

    #[tokio::test]
    async fn list_products_with_category_id_filter() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state);
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "B",
            "name": "P"
        });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/products?category_id={cat_id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let arr = json.as_array().expect("array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].get("name").and_then(|v| v.as_str()), Some("P"));
    }

    #[tokio::test]
    async fn list_products_with_q_search() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state);
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "DairyCo",
            "name": "Milk"
        });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/products?q=Milk")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let arr = json.as_array().expect("array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].get("name").and_then(|v| v.as_str()), Some("Milk"));
    }

    #[tokio::test]
    async fn update_product_persists_new_values() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "Old",
            "name": "Prod"
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let bytes = create_resp
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let created: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let id = created.get("id").and_then(|v| v.as_str()).expect("id");
        let patch_body = serde_json::json!({ "brand": "NewBrand", "name": "NewName" });
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/products/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(json.get("brand").and_then(|v| v.as_str()), Some("NewBrand"));
        assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("NewName"));
        let uuid = Uuid::parse_str(id).expect("uuid");
        let persisted = db::product::get_by_id(&state.pool, uuid).await.expect("db");
        let persisted = persisted.expect("product in db");
        assert_eq!(persisted.brand(), "NewBrand");
        assert_eq!(persisted.name(), "NewName");
    }

    #[tokio::test]
    async fn update_product_returns_404_when_product_not_found() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let id = Uuid::new_v4();
        let patch_body = serde_json::json!({ "name": "New" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/products/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn update_product_returns_200_when_no_change_does_not_persist() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state);
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "B",
            "name": "P"
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let bytes = create_resp
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let created: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let id = created.get("id").and_then(|v| v.as_str()).expect("id");
        let patch_body = serde_json::json!({ "brand": "B", "name": "P" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/products/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("P"));
    }

    #[tokio::test]
    async fn update_product_returns_404_when_category_not_found() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state);
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "B",
            "name": "P"
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let bytes = create_resp
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let created: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let id = created.get("id").and_then(|v| v.as_str()).expect("id");
        let fake_cat_id = Uuid::new_v4();
        let patch_body = serde_json::json!({ "category_id": fake_cat_id.to_string() });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/products/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_product_returns_204_soft_delete() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "B",
            "name": "ToDelete"
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let bytes = create_resp
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let created: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let id = created.get("id").and_then(|v| v.as_str()).expect("id");
        let uuid = Uuid::parse_str(id).expect("uuid");
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/products/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let active = db::product::get_by_id(&state.pool, uuid).await.expect("db");
        assert!(active.is_none(), "get_by_id must exclude soft-deleted");
        let with_deleted = db::product::get_all_with_deleted(&state.pool)
            .await
            .expect("db");
        let soft_deleted = with_deleted
            .iter()
            .find(|p| p.id() == uuid)
            .expect("row in db");
        assert!(
            soft_deleted.deleted_at().is_some(),
            "deleted_at must be set"
        );
    }

    #[tokio::test]
    async fn delete_product_hard_remove_row() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "B",
            "name": "ToHardDelete"
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let bytes = create_resp
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let created: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let id = created.get("id").and_then(|v| v.as_str()).expect("id");
        let uuid = Uuid::parse_str(id).expect("uuid");
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/products/{id}?force=true"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let active = db::product::get_by_id(&state.pool, uuid).await.expect("db");
        assert!(active.is_none());
        let with_deleted = db::product::get_all_with_deleted(&state.pool)
            .await
            .expect("db");
        assert!(
            !with_deleted.iter().any(|p| p.id() == uuid),
            "hard delete must remove row"
        );
    }

    #[tokio::test]
    async fn delete_product_returns_404_when_not_found() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/products/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_product_returns_409_when_has_purchases() {
        let (state, _dir) = test_pool().await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({
            "category_id": cat_id.to_string(),
            "brand": "B",
            "name": "WithPurchase"
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let bytes = create_resp
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let created: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let product_id =
            Uuid::parse_str(created.get("id").and_then(|v| v.as_str()).expect("id")).expect("uuid");
        let now = chrono::Utc::now().timestamp();
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
        .execute(&state.pool)
        .await
        .expect("insert user");
        sqlx::query("INSERT INTO locations (id, name, deleted_at) VALUES (?, ?, ?)")
            .bind(location_id.to_string())
            .bind("Store")
            .bind::<Option<i64>>(None)
            .execute(&state.pool)
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
        .execute(&state.pool)
        .await
        .expect("insert purchase");
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/products/{product_id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
