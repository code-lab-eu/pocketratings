//! Reviews REST API: list, get, create, update, delete.

use std::str::FromStr;

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
use crate::domain::review::{Review, ValidationError};

/// Request body for creating a review. Ignores `id`, `user_id`, `created_at`, `updated_at`, `deleted_at`.
#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub product_id: Uuid,
    /// Rating 1–5 (JSON number); converted to Decimal.
    pub rating: f64,
    pub text: Option<String>,
}

/// Request body for partial update.
#[derive(Debug, Deserialize)]
pub struct UpdateReviewRequest {
    /// Rating 1–5 (JSON number); converted to Decimal.
    pub rating: Option<f64>,
    pub text: Option<String>,
}

/// Query params for list reviews.
#[derive(Debug, Default, Deserialize)]
pub struct ListReviewsQuery {
    pub product_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

/// Query params for delete (optional force).
#[derive(Debug, Default, Deserialize)]
pub struct DeleteReviewQuery {
    #[serde(default, deserialize_with = "parse_force")]
    pub force: bool,
}

/// Response body: review with timestamps as i64, rating as number.
#[derive(Debug, serde::Serialize)]
pub struct ReviewResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub user_id: Uuid,
    #[serde(serialize_with = "serialize_rating")]
    pub rating: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

fn serialize_rating<S>(d: &Decimal, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use rust_decimal::prelude::ToPrimitive;
    s.serialize_f64(d.to_f64().unwrap_or(0.0))
}

fn review_to_response(r: &Review) -> ReviewResponse {
    ReviewResponse {
        id: r.id(),
        product_id: r.product_id(),
        user_id: r.user_id(),
        rating: r.rating(),
        text: r.text().map(std::string::ToString::to_string),
        created_at: r.created_at(),
        updated_at: r.updated_at(),
        deleted_at: r.deleted_at(),
    }
}

/// Map `DbError` to `ApiError` for review operations.
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

/// Deserialize "true" / "false" for force query param.
fn parse_force<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
    Ok(s.to_lowercase() == "true" || s == "1")
}

/// GET /api/v1/reviews — list reviews; default `user_id` = current user ("my reviews").
pub async fn list_reviews(
    State(state): State<AppState>,
    Extension(CurrentUserId(current_user_id)): Extension<CurrentUserId>,
    Query(q): Query<ListReviewsQuery>,
) -> Result<Json<Vec<ReviewResponse>>, ApiError> {
    let user_id = q.user_id.or(Some(current_user_id));
    let list = db::review::list(&state.pool, q.product_id, user_id, false)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok(Json(list.iter().map(review_to_response).collect()))
}

/// GET /api/v1/reviews/:id — get one review.
pub async fn get_review(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let review = db::review::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let review = review.ok_or_else(|| ApiError::NotFound("review not found".to_string()))?;
    Ok(Json(review_to_response(&review)))
}

/// POST /api/v1/reviews — create a review (`user_id` from JWT).
pub async fn create_review(
    State(state): State<AppState>,
    Extension(CurrentUserId(user_id)): Extension<CurrentUserId>,
    Json(body): Json<CreateReviewRequest>,
) -> Result<(StatusCode, Json<ReviewResponse>), ApiError> {
    let product = db::product::get_by_id(&state.pool, body.product_id)
        .await
        .map_err(|e| map_db_error(&e))?;
    if product.is_none() {
        return Err(ApiError::NotFound("product not found".to_string()));
    }

    let Ok(rating) = Decimal::from_str(&body.rating.to_string()) else {
        return Err(ApiError::BadRequest("invalid rating".to_string()));
    };
    let now = chrono::Utc::now().timestamp();
    let id = Uuid::new_v4();
    let review = Review::new(
        id,
        body.product_id,
        user_id,
        rating,
        body.text,
        now,
        now,
        None,
    )
    .map_err(|e: ValidationError| {
        ApiError::BadRequest(match &e {
            ValidationError::RatingOutOfRange { rating } => {
                format!("rating must be between 1 and 5 (got {rating})")
            }
            _ => e.to_string(),
        })
    })?;
    db::review::insert(&state.pool, &review)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok((StatusCode::CREATED, Json(review_to_response(&review))))
}

/// PATCH /api/v1/reviews/:id — partial update; only owner; only persist if changed.
pub async fn update_review(
    State(state): State<AppState>,
    Extension(CurrentUserId(current_user_id)): Extension<CurrentUserId>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateReviewRequest>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let existing = db::review::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let existing = existing.ok_or_else(|| ApiError::NotFound("review not found".to_string()))?;

    if existing.user_id() != current_user_id {
        return Err(ApiError::Forbidden(
            "not allowed to update another user's review".to_string(),
        ));
    }

    let rating = body.rating.map_or_else(
        || existing.rating(),
        |f| Decimal::from_str(&f.to_string()).unwrap_or(Decimal::ZERO),
    );
    let text = body
        .text
        .or_else(|| existing.text().map(std::string::ToString::to_string));

    if existing.rating() == rating && existing.text() == text.as_deref() {
        return Ok(Json(review_to_response(&existing)));
    }

    let updated = Review::new(
        existing.id(),
        existing.product_id(),
        existing.user_id(),
        rating,
        text,
        existing.created_at(),
        chrono::Utc::now().timestamp(),
        existing.deleted_at(),
    )
    .map_err(|e: ValidationError| {
        ApiError::BadRequest(match &e {
            ValidationError::RatingOutOfRange { rating: r } => {
                format!("rating must be between 1 and 5 (got {r})")
            }
            _ => e.to_string(),
        })
    })?;
    db::review::update(&state.pool, &updated)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok(Json(review_to_response(&updated)))
}

/// DELETE /api/v1/reviews/:id — soft delete, or hard with ?force=true; only owner.
pub async fn delete_review(
    State(state): State<AppState>,
    Extension(CurrentUserId(current_user_id)): Extension<CurrentUserId>,
    Path(id): Path<Uuid>,
    Query(q): Query<DeleteReviewQuery>,
) -> Result<StatusCode, ApiError> {
    let review = db::review::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let review = review.ok_or_else(|| ApiError::NotFound("review not found".to_string()))?;

    if review.user_id() != current_user_id {
        return Err(ApiError::Forbidden(
            "not allowed to delete another user's review".to_string(),
        ));
    }

    if q.force {
        db::review::hard_delete(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?;
    } else {
        db::review::soft_delete(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Router for /api/v1/reviews (all five handlers).
pub fn route() -> Router<AppState> {
    Router::new()
        .route("/api/v1/reviews", get(list_reviews).post(create_review))
        .route(
            "/api/v1/reviews/{id}",
            get(get_review).patch(update_review).delete(delete_review),
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
    use crate::test_support::{insert_category, insert_product, insert_user};
    use sqlx::SqlitePool;

    /// Build the review route with a fixed current user (no auth header needed). Same pattern as category/location/product tests.
    fn app_with_user(state: AppState, user_id: Uuid) -> axum::Router {
        route()
            .layer(Extension(CurrentUserId(user_id)))
            .with_state(state)
    }

    async fn test_pool() -> (AppState, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("review_test.db");
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
                    .join("pocketratings-review-test.pid")
                    .to_string_lossy()
                    .into_owned(),
            },
            pool,
        };
        (state, dir)
    }

    #[tokio::test]
    async fn list_reviews_returns_empty_array_when_none() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "Alice", "a@example.com").await;
        let app = app_with_user(state, user_id);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/reviews")
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
    async fn list_reviews_with_product_id_filter() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let now = chrono::Utc::now().timestamp();
        let review_id = Uuid::new_v4();
        let review = Review::new(
            review_id,
            product_id,
            user_id,
            Decimal::from(4),
            Some("nice".to_string()),
            now,
            now,
            None,
        )
        .expect("valid");
        db::review::insert(&state.pool, &review)
            .await
            .expect("insert");
        let app = app_with_user(state, user_id);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/reviews?product_id={}", product_id))
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
        assert_eq!(
            arr[0].get("product_id").and_then(|v| v.as_str()),
            Some(product_id.to_string().as_str())
        );
        assert_eq!(arr[0].get("rating").and_then(|v| v.as_f64()), Some(4.0));
    }

    #[tokio::test]
    async fn list_reviews_defaults_to_current_user() {
        let (state, _dir) = test_pool().await;
        let user_a = insert_user(&state.pool, "A", "a@ex.com").await;
        let user_b = insert_user(&state.pool, "B", "b@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let now = chrono::Utc::now().timestamp();
        let review_a = Review::new(
            Uuid::new_v4(),
            product_id,
            user_a,
            Decimal::from(3),
            None,
            now,
            now,
            None,
        )
        .expect("valid");
        let review_b = Review::new(
            Uuid::new_v4(),
            product_id,
            user_b,
            Decimal::from(5),
            None,
            now,
            now,
            None,
        )
        .expect("valid");
        db::review::insert(&state.pool, &review_a)
            .await
            .expect("insert");
        db::review::insert(&state.pool, &review_b)
            .await
            .expect("insert");
        let app = app_with_user(state, user_a);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/reviews")
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
        assert_eq!(arr.len(), 1, "only current user's review");
        assert_eq!(
            arr[0].get("user_id").and_then(|v| v.as_str()),
            Some(user_a.to_string().as_str())
        );
    }

    #[tokio::test]
    async fn list_reviews_with_explicit_user_id() {
        let (state, _dir) = test_pool().await;
        let user_a = insert_user(&state.pool, "A", "a@ex.com").await;
        let user_b = insert_user(&state.pool, "B", "b@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let now = chrono::Utc::now().timestamp();
        let review_b = Review::new(
            Uuid::new_v4(),
            product_id,
            user_b,
            Decimal::from(5),
            Some("great".to_string()),
            now,
            now,
            None,
        )
        .expect("valid");
        db::review::insert(&state.pool, &review_b)
            .await
            .expect("insert");
        let app = app_with_user(state, user_a);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/reviews?user_id={}", user_b))
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
        assert_eq!(
            arr[0].get("user_id").and_then(|v| v.as_str()),
            Some(user_b.to_string().as_str())
        );
    }

    #[tokio::test]
    async fn list_reviews_invalid_uuid_returns_400() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let app = app_with_user(state, user_id);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/reviews?product_id=not-a-uuid")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_review_returns_200_with_correct_body() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let app = app_with_user(state.clone(), user_id);
        let body = serde_json::json!({
            "product_id": product_id.to_string(),
            "rating": 4.5,
            "text": "Good."
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/reviews")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(create_resp.status(), StatusCode::CREATED);
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
                    .uri(format!("/api/v1/reviews/{id}"))
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
        assert_eq!(
            json.get("rating").and_then(serde_json::Value::as_f64),
            Some(4.5)
        );
        assert_eq!(json.get("text").and_then(|v| v.as_str()), Some("Good."));
    }

    #[tokio::test]
    async fn get_review_returns_404_when_not_found() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let app = app_with_user(state, user_id);
        let id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/reviews/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn create_review_returns_201_and_persists() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "Acme", "Widget").await;
        let app = app_with_user(state.clone(), user_id);
        let body = serde_json::json!({
            "product_id": product_id.to_string(),
            "rating": 4,
            "text": "Nice product"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/reviews")
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
        assert_eq!(
            json.get("product_id").and_then(|v| v.as_str()),
            Some(product_id.to_string().as_str())
        );
        assert_eq!(
            json.get("user_id").and_then(|v| v.as_str()),
            Some(user_id.to_string().as_str())
        );
        assert_eq!(
            json.get("rating").and_then(serde_json::Value::as_f64),
            Some(4.0)
        );
        assert_eq!(
            json.get("text").and_then(|v| v.as_str()),
            Some("Nice product")
        );
        let id_str = json.get("id").and_then(|v| v.as_str()).expect("id");
        let id = Uuid::parse_str(id_str).expect("uuid");
        let persisted = db::review::get_by_id(&state.pool, id).await.expect("db");
        let persisted = persisted.expect("review in db");
        assert_eq!(persisted.product_id(), product_id);
        assert_eq!(persisted.user_id(), user_id);
        assert_eq!(persisted.rating(), Decimal::from(4));
        assert_eq!(persisted.text(), Some("Nice product"));
    }

    #[tokio::test]
    async fn create_review_returns_404_when_product_not_found() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let app = app_with_user(state, user_id);
        let fake_product_id = Uuid::new_v4();
        let body = serde_json::json!({
            "product_id": fake_product_id.to_string(),
            "rating": 3,
            "text": null
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/reviews")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn create_review_returns_400_when_rating_out_of_range() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let app = app_with_user(state.clone(), user_id);
        for (rating, _label) in [(0.0, "zero"), (6.0, "six")] {
            let body = serde_json::json!({
                "product_id": product_id.to_string(),
                "rating": rating,
                "text": null
            });
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/reviews")
                        .header("content-type", "application/json")
                        .body(Body::from(serde_json::to_vec(&body).expect("json")))
                        .expect("request"),
                )
                .await
                .expect("service");
            assert_eq!(
                response.status(),
                StatusCode::BAD_REQUEST,
                "rating {} should be rejected",
                rating
            );
        }
    }

    #[tokio::test]
    async fn update_review_returns_200_and_persists() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let app = app_with_user(state.clone(), user_id);
        let body = serde_json::json!({
            "product_id": product_id.to_string(),
            "rating": 3,
            "text": "Original"
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/reviews")
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
        let patch_body = serde_json::json!({ "rating": 5, "text": "Updated" });
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/reviews/{id}"))
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
        assert_eq!(
            json.get("rating").and_then(serde_json::Value::as_f64),
            Some(5.0)
        );
        assert_eq!(json.get("text").and_then(|v| v.as_str()), Some("Updated"));
        let uuid = Uuid::parse_str(id).expect("uuid");
        let persisted = db::review::get_by_id(&state.pool, uuid).await.expect("db");
        let persisted = persisted.expect("review in db");
        assert_eq!(persisted.rating(), Decimal::from(5));
        assert_eq!(persisted.text(), Some("Updated"));
    }

    #[tokio::test]
    async fn update_review_returns_404_when_not_found() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let app = app_with_user(state, user_id);
        let id = Uuid::new_v4();
        let patch_body = serde_json::json!({ "rating": 4 });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/reviews/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn update_review_returns_403_when_not_owner() {
        let (state, _dir) = test_pool().await;
        let user_a = insert_user(&state.pool, "A", "a@ex.com").await;
        let user_b = insert_user(&state.pool, "B", "b@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let now = chrono::Utc::now().timestamp();
        let review = Review::new(
            Uuid::new_v4(),
            product_id,
            user_a,
            Decimal::from(4),
            None,
            now,
            now,
            None,
        )
        .expect("valid");
        db::review::insert(&state.pool, &review)
            .await
            .expect("insert");
        let review_id = review.id();
        let app = app_with_user(state, user_b);
        let patch_body = serde_json::json!({ "rating": 1 });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/reviews/{review_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn update_review_returns_400_when_rating_out_of_range() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let app = app_with_user(state.clone(), user_id);
        let body = serde_json::json!({
            "product_id": product_id.to_string(),
            "rating": 4,
            "text": null
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/reviews")
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
        let patch_body = serde_json::json!({ "rating": 0 });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/reviews/{}", id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_review_no_op_returns_200_without_db_update() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let app = app_with_user(state.clone(), user_id);
        let body = serde_json::json!({
            "product_id": product_id.to_string(),
            "rating": 4,
            "text": "Same"
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/reviews")
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
        let updated_at_before = created
            .get("updated_at")
            .and_then(serde_json::Value::as_i64)
            .expect("updated_at");
        let patch_body = serde_json::json!({ "rating": 4, "text": "Same" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/reviews/{id}"))
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
        assert_eq!(
            json.get("updated_at").and_then(serde_json::Value::as_i64),
            Some(updated_at_before)
        );
    }

    #[tokio::test]
    async fn delete_review_soft_sets_deleted_at_and_excludes_from_list() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let app = app_with_user(state.clone(), user_id);
        let body = serde_json::json!({
            "product_id": product_id.to_string(),
            "rating": 4,
            "text": null
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/reviews")
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
                    .uri(format!("/api/v1/reviews/{}", id))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let get_result = db::review::get_by_id(&state.pool, uuid).await.expect("db");
        assert!(get_result.is_none(), "get_by_id must exclude soft-deleted");
        let list = db::review::list(&state.pool, None, Some(user_id), false)
            .await
            .expect("db");
        assert!(!list.iter().any(|r| r.id() == uuid));
        let with_deleted = db::review::list(&state.pool, None, Some(user_id), true)
            .await
            .expect("db");
        let soft_deleted = with_deleted
            .iter()
            .find(|r| r.id() == uuid)
            .expect("row still in db");
        assert!(soft_deleted.deleted_at().is_some());
    }

    #[tokio::test]
    async fn delete_review_force_removes_row() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let app = app_with_user(state.clone(), user_id);
        let body = serde_json::json!({
            "product_id": product_id.to_string(),
            "rating": 4,
            "text": null
        });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/reviews")
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
                    .uri(format!("/api/v1/reviews/{id}?force=true"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let list_all = db::review::list(&state.pool, None, None, true)
            .await
            .expect("db");
        assert!(!list_all.iter().any(|r| r.id() == uuid));
    }

    #[tokio::test]
    async fn delete_review_returns_404_when_not_found() {
        let (state, _dir) = test_pool().await;
        let user_id = insert_user(&state.pool, "U", "u@ex.com").await;
        let app = app_with_user(state, user_id);
        let id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/reviews/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_review_returns_403_when_not_owner() {
        let (state, _dir) = test_pool().await;
        let user_a = insert_user(&state.pool, "A", "a@ex.com").await;
        let user_b = insert_user(&state.pool, "B", "b@ex.com").await;
        let cat_id = insert_category(&state.pool, "Cat").await;
        let product_id = insert_product(&state.pool, cat_id, "B", "P").await;
        let now = chrono::Utc::now().timestamp();
        let review = Review::new(
            Uuid::new_v4(),
            product_id,
            user_a,
            Decimal::from(4),
            None,
            now,
            now,
            None,
        )
        .expect("valid");
        db::review::insert(&state.pool, &review)
            .await
            .expect("insert");
        let review_id = review.id();
        let app = app_with_user(state, user_b);
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/reviews/{review_id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
