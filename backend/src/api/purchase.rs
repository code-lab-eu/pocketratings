//! Purchases REST API: list, get, create, update, delete.
//!
//! List endpoints return `200 OK` with an empty array when there are no matching
//! records (e.g. product exists but has no purchases). They do not return 404.

use axum::extract::Extension;
use axum::routing::get;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::api::auth::CurrentUserId;
use crate::api::{error::ApiError, state::AppState};
use crate::db;
use crate::domain::purchase::{Purchase, ValidationError};

/// Query params for list purchases.
#[derive(Debug, Default, Deserialize)]
pub struct ListPurchasesQuery {
    pub user_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// Request body for creating a purchase.
#[derive(Debug, Deserialize)]
pub struct CreatePurchaseRequest {
    pub product_id: Uuid,
    pub location_id: Uuid,
    #[serde(default)]
    pub quantity: Option<i32>,
    pub price: String,
    pub purchased_at: Option<String>,
}

/// Request body for partial update.
#[derive(Debug, Deserialize)]
pub struct UpdatePurchaseRequest {
    pub product_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub quantity: Option<i32>,
    pub price: Option<String>,
    pub purchased_at: Option<String>,
}

/// Query params for delete (optional force).
#[derive(Debug, Default, Deserialize)]
pub struct DeletePurchaseQuery {
    #[serde(default, deserialize_with = "parse_force")]
    pub force: bool,
}

fn parse_force<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
    Ok(s.to_lowercase() == "true" || s == "1")
}

/// Response body: purchase with price as string, timestamps as i64.
#[derive(Debug, serde::Serialize)]
pub struct PurchaseResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub quantity: i32,
    pub price: String,
    pub purchased_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

fn purchase_to_response(p: &Purchase) -> PurchaseResponse {
    PurchaseResponse {
        id: p.id(),
        user_id: p.user_id(),
        product_id: p.product_id(),
        location_id: p.location_id(),
        quantity: p.quantity(),
        price: p.price().to_string(),
        purchased_at: p.purchased_at(),
        deleted_at: p.deleted_at(),
    }
}

/// Map `DbError` to `ApiError` for purchase operations.
fn map_db_error(e: &db::DbError) -> ApiError {
    match e {
        db::DbError::InvalidData(msg) => ApiError::BadRequest(msg.clone()),
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

/// Parse ISO 8601 date string to UNIX timestamp. Returns None if invalid or missing.
fn parse_iso_date_to_ts(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp())
}

/// GET /api/v1/purchases — list purchases; default `user_id` = current user.
/// Returns 200 with an empty array when there are no matching purchases (e.g. product has none).
pub async fn list_purchases(
    State(state): State<AppState>,
    Extension(CurrentUserId(current_user_id)): Extension<CurrentUserId>,
    Query(q): Query<ListPurchasesQuery>,
) -> Result<Json<Vec<PurchaseResponse>>, ApiError> {
    let user_id = q.user_id.or(Some(current_user_id));
    let from_ts = q.from.as_deref().and_then(parse_iso_date_to_ts);
    let to_ts = q.to.as_deref().and_then(parse_iso_date_to_ts);
    let list = db::purchase::list(
        &state.pool,
        user_id,
        q.product_id,
        q.location_id,
        from_ts,
        to_ts,
        false,
    )
    .await
    .map_err(|e| map_db_error(&e))?;
    Ok(Json(list.iter().map(purchase_to_response).collect()))
}

/// GET /api/v1/purchases/:id — get one purchase.
pub async fn get_purchase(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PurchaseResponse>, ApiError> {
    let purchase = db::purchase::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let purchase = purchase.ok_or_else(|| ApiError::NotFound("purchase not found".to_string()))?;
    Ok(Json(purchase_to_response(&purchase)))
}

/// POST /api/v1/purchases — create a purchase (`user_id` from JWT).
pub async fn create_purchase(
    State(state): State<AppState>,
    Extension(CurrentUserId(user_id)): Extension<CurrentUserId>,
    Json(body): Json<CreatePurchaseRequest>,
) -> Result<(StatusCode, Json<PurchaseResponse>), ApiError> {
    let product = db::product::get_by_id(&state.pool, body.product_id)
        .await
        .map_err(|e| map_db_error(&e))?;
    if product.is_none() {
        return Err(ApiError::NotFound("product not found".to_string()));
    }
    let location = db::location::get_by_id(&state.pool, body.location_id)
        .await
        .map_err(|e| map_db_error(&e))?;
    if location.is_none() {
        return Err(ApiError::NotFound("location not found".to_string()));
    }

    let quantity = body.quantity.unwrap_or(1);
    let price: Decimal = body
        .price
        .parse()
        .map_err(|_| ApiError::BadRequest("invalid price".to_string()))?;
    let purchased_at = body
        .purchased_at
        .as_deref()
        .and_then(parse_iso_date_to_ts)
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    let id = Uuid::new_v4();
    let purchase = Purchase::new(
        id,
        user_id,
        body.product_id,
        body.location_id,
        quantity,
        price,
        purchased_at,
        None,
    )
    .map_err(|e: ValidationError| {
        ApiError::BadRequest(match &e {
            ValidationError::QuantityInvalid { quantity: q } => {
                format!("quantity must be at least 1 (got {q})")
            }
            ValidationError::PriceInvalid { .. } => "price must not be negative".to_string(),
        })
    })?;
    db::purchase::insert(&state.pool, &purchase)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok((StatusCode::CREATED, Json(purchase_to_response(&purchase))))
}

/// PATCH /api/v1/purchases/:id — partial update; only owner.
pub async fn update_purchase(
    State(state): State<AppState>,
    Extension(CurrentUserId(current_user_id)): Extension<CurrentUserId>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePurchaseRequest>,
) -> Result<Json<PurchaseResponse>, ApiError> {
    let existing = db::purchase::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let existing = existing.ok_or_else(|| ApiError::NotFound("purchase not found".to_string()))?;

    if existing.user_id() != current_user_id {
        return Err(ApiError::Forbidden(
            "not allowed to update another user's purchase".to_string(),
        ));
    }

    let product_id = body.product_id.unwrap_or_else(|| existing.product_id());
    let location_id = body.location_id.unwrap_or_else(|| existing.location_id());
    let quantity = body.quantity.unwrap_or_else(|| existing.quantity());
    let price = body
        .price
        .as_ref()
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or_else(|| existing.price());
    let purchased_at = body
        .purchased_at
        .as_deref()
        .and_then(parse_iso_date_to_ts)
        .unwrap_or_else(|| existing.purchased_at());

    let updated = Purchase::new(
        existing.id(),
        existing.user_id(),
        product_id,
        location_id,
        quantity,
        price,
        purchased_at,
        existing.deleted_at(),
    )
    .map_err(|e: ValidationError| {
        ApiError::BadRequest(match &e {
            ValidationError::QuantityInvalid { .. } => "quantity must be at least 1".to_string(),
            ValidationError::PriceInvalid { .. } => "price must not be negative".to_string(),
        })
    })?;

    if body.product_id.is_some()
        && db::product::get_by_id(&state.pool, product_id)
            .await
            .ok()
            .flatten()
            .is_none()
    {
        return Err(ApiError::NotFound("product not found".to_string()));
    }
    if body.location_id.is_some()
        && db::location::get_by_id(&state.pool, location_id)
            .await
            .ok()
            .flatten()
            .is_none()
    {
        return Err(ApiError::NotFound("location not found".to_string()));
    }

    db::purchase::update(&state.pool, &updated)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok(Json(purchase_to_response(&updated)))
}

/// DELETE /api/v1/purchases/:id — soft delete, or hard with ?force=true; only owner.
pub async fn delete_purchase(
    State(state): State<AppState>,
    Extension(CurrentUserId(current_user_id)): Extension<CurrentUserId>,
    Path(id): Path<Uuid>,
    Query(q): Query<DeletePurchaseQuery>,
) -> Result<StatusCode, ApiError> {
    let purchase = db::purchase::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let purchase = purchase.ok_or_else(|| ApiError::NotFound("purchase not found".to_string()))?;

    if purchase.user_id() != current_user_id {
        return Err(ApiError::Forbidden(
            "not allowed to delete another user's purchase".to_string(),
        ));
    }

    if q.force {
        db::purchase::hard_delete(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?;
    } else {
        db::purchase::soft_delete(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Router for /api/v1/purchases.
pub fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/purchases",
            get(list_purchases).post(create_purchase),
        )
        .route(
            "/api/v1/purchases/{id}",
            get(get_purchase)
                .patch(update_purchase)
                .delete(delete_purchase),
        )
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::Extension;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use rust_decimal::Decimal;
    use tower::ServiceExt;

    use super::*;
    use crate::api::auth::CurrentUserId;
    use crate::config::Config;
    use crate::db;
    use crate::test_helpers::{insert_category, insert_location, insert_product, insert_user};

    /// Build the purchase route with a fixed current user (no auth header needed).
    fn app_with_user(state: AppState, user_id: Uuid) -> axum::Router {
        route()
            .layer(Extension(CurrentUserId(user_id)))
            .with_state(state)
    }

    async fn test_pool() -> (AppState, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("purchase_api_test.db");
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
                    .join("pocketratings-purchase-api-test.pid")
                    .to_string_lossy()
                    .into_owned(),
            },
            pool,
        };
        (state, dir)
    }

    #[tokio::test]
    async fn list_purchases_returns_empty_array_when_none() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "Alice", "a@example.com").await;
        let app = app_with_user(state, user_id);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/purchases")
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
    async fn create_and_get_purchase_roundtrip() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "Bob", "b@example.com").await;
        let category_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, category_id, "Brand", "Name").await;
        let location_id = insert_location(&state.pool, "Store").await;
        let app = app_with_user(state, user_id);

        let body = serde_json::json!({
            "product_id": product_id,
            "location_id": location_id,
            "quantity": 2,
            "price": "3.50",
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/purchases")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
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
        let created: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let created_id = created
            .get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .expect("created id");

        // GET by id returns the same purchase.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/purchases/{created_id}"))
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
        let got: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(
            got.get("user_id").and_then(|v| v.as_str()),
            Some(user_id.to_string().as_str())
        );
        assert_eq!(
            got.get("product_id").and_then(|v| v.as_str()),
            Some(product_id.to_string().as_str())
        );
        assert_eq!(
            got.get("location_id").and_then(|v| v.as_str()),
            Some(location_id.to_string().as_str())
        );
        assert_eq!(
            got.get("quantity").and_then(serde_json::Value::as_i64),
            Some(2)
        );
    }

    #[tokio::test]
    async fn create_purchase_rejects_invalid_price() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "Bob", "b@example.com").await;
        let category_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, category_id, "Brand", "Name").await;
        let location_id = insert_location(&state.pool, "Store").await;
        let app = app_with_user(state, user_id);

        let body = serde_json::json!({
            "product_id": product_id,
            "location_id": location_id,
            "quantity": 1,
            "price": "not-a-number",
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/purchases")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .expect("request"),
            )
            .await
            .expect("service");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_nonexistent_purchase_returns_404() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "Bob", "b@example.com").await;
        let app = app_with_user(state, user_id);
        let missing_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/purchases/{missing_id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn update_purchase_forbidden_for_other_user() {
        let (state, _dir) = test_pool().await;
        let owner = insert_user(&state.pool, "Owner", "o@example.com").await;
        let other = insert_user(&state.pool, "Other", "x@example.com").await;
        let category_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, category_id, "Brand", "Name").await;
        let location_id = insert_location(&state.pool, "Store").await;

        let now = chrono::Utc::now().timestamp();
        let purchase = Purchase::new(
            Uuid::new_v4(),
            owner,
            product_id,
            location_id,
            1,
            Decimal::from(2),
            now,
            None,
        )
        .expect("valid purchase");
        db::purchase::insert(&state.pool, &purchase)
            .await
            .expect("insert purchase");

        let app = app_with_user(state, other);
        let body = serde_json::json!({ "quantity": 3 });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/purchases/{}", purchase.id()))
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .expect("request"),
            )
            .await
            .expect("service");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn delete_purchase_forbidden_for_other_user() {
        let (state, _dir) = test_pool().await;
        let owner = insert_user(&state.pool, "Owner", "o@example.com").await;
        let other = insert_user(&state.pool, "Other", "x@example.com").await;
        let category_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, category_id, "Brand", "Name").await;
        let location_id = insert_location(&state.pool, "Store").await;

        let now = chrono::Utc::now().timestamp();
        let purchase = Purchase::new(
            Uuid::new_v4(),
            owner,
            product_id,
            location_id,
            1,
            Decimal::from(2),
            now,
            None,
        )
        .expect("valid purchase");
        db::purchase::insert(&state.pool, &purchase)
            .await
            .expect("insert purchase");

        let app = app_with_user(state, other);
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/purchases/{}", purchase.id()))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
