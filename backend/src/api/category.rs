//! Categories REST API: list, get, create, update, delete.

use axum::routing::get;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::{error::ApiError, state::AppState};
use crate::db;
use crate::domain::category::Category;

/// Request body for creating a category.
#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

/// Request body for partial update.
#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub parent_id: Option<Uuid>,
}

/// Query params for list categories.
#[derive(Debug, Default, Deserialize)]
pub struct ListCategoriesQuery {
    pub parent_id: Option<Uuid>,
    /// When `Some(1)`, return only one level (roots when no `parent_id`, or direct children of `parent_id`). Omit for full tree.
    #[serde(default)]
    pub depth: Option<u8>,
}

/// Query params for delete (optional force).
#[derive(Debug, Default, Deserialize)]
pub struct DeleteCategoryQuery {
    #[serde(default, deserialize_with = "parse_force")]
    pub force: bool,
}

/// Response body: category with timestamps as i64 and optional nested children (list endpoint only).
#[derive(Debug, serde::Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
    /// Nested children.
    pub children: Vec<Self>,
}

fn category_to_response(c: &Category) -> CategoryResponse {
    CategoryResponse {
        id: c.id(),
        parent_id: c.parent_id(),
        name: c.name().to_string(),
        created_at: c.created_at(),
        updated_at: c.updated_at(),
        deleted_at: c.deleted_at(),
        children: Vec::new(),
    }
}

/// Map a slice of tree nodes to response list (recursive).
fn categories_to_response_list(nodes: &[db::category::Categories]) -> Vec<CategoryResponse> {
    nodes
        .iter()
        .filter_map(|n| {
            let c = n.category.as_ref()?;
            Some(CategoryResponse {
                id: c.id(),
                parent_id: c.parent_id(),
                name: c.name().to_string(),
                created_at: c.created_at(),
                updated_at: c.updated_at(),
                deleted_at: c.deleted_at(),
                children: categories_to_response_list(&n.children),
            })
        })
        .collect()
}

/// Map `DbError` to `ApiError` for category operations.
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

/// GET /api/v1/categories — list categories as a nested tree; optional `parent_id` and `depth`.
pub async fn list_categories(
    State(state): State<AppState>,
    Query(q): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let depth = q.depth.filter(|&d| d > 0);

    let response = if depth == Some(1) {
        let list = db::category::get_children(&state.pool, q.parent_id)
            .await
            .map_err(|e| map_db_error(&e))?;
        list.into_iter()
            .map(|c| CategoryResponse {
                id: c.id(),
                parent_id: c.parent_id(),
                name: c.name().to_string(),
                created_at: c.created_at(),
                updated_at: c.updated_at(),
                deleted_at: c.deleted_at(),
                children: Vec::new(),
            })
            .collect()
    } else {
        let all = db::category::get_all(&state.pool, false)
            .await
            .map_err(|e| map_db_error(&e))?;
        let root = if let Some(pid) = q.parent_id {
            let parent = db::category::get_by_id(&state.pool, pid)
                .await
                .map_err(|e| map_db_error(&e))?;
            let parent = parent
                .ok_or_else(|| ApiError::NotFound("parent category not found".to_string()))?;
            Some(parent)
        } else {
            None
        };
        let tree = db::category::Categories::from_list(all, root, depth, false);
        categories_to_response_list(&tree.children)
    };

    Ok(Json(response))
}

/// GET /api/v1/categories/:id — get one category.
pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let category = db::category::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let category = category.ok_or_else(|| ApiError::NotFound("category not found".to_string()))?;
    Ok(Json(category_to_response(&category)))
}

/// POST /api/v1/categories — create a category.
pub async fn create_category(
    State(state): State<AppState>,
    Json(body): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CategoryResponse>), ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name must not be empty".to_string()));
    }
    if let Some(pid) = body.parent_id {
        let parent = db::category::get_by_id(&state.pool, pid)
            .await
            .map_err(|_| ApiError::Internal)?;
        if parent.is_none() {
            return Err(ApiError::NotFound("parent category not found".to_string()));
        }
    }
    let now = chrono::Utc::now().timestamp();
    let id = Uuid::new_v4();
    let category = Category::new(
        id,
        body.parent_id,
        body.name.trim().to_string(),
        now,
        now,
        None,
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    db::category::insert(&state.pool, &category)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok((StatusCode::CREATED, Json(category_to_response(&category))))
}

/// PATCH /api/v1/categories/:id — partial update; only persist if something changed.
pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let existing = db::category::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let existing = existing.ok_or_else(|| ApiError::NotFound("category not found".to_string()))?;

    let new_name = body
        .name
        .as_deref()
        .map(str::trim)
        .map(std::string::ToString::to_string);
    let new_parent_id = body.parent_id;

    let (name, parent_id) = (
        new_name.unwrap_or_else(|| existing.name().to_string()),
        new_parent_id.or_else(|| existing.parent_id()),
    );

    if let Some(pid) = parent_id {
        if pid == id {
            return Err(ApiError::BadRequest(
                "parent_id cannot be the category itself".to_string(),
            ));
        }
        let parent = db::category::get_by_id(&state.pool, pid)
            .await
            .map_err(|_| ApiError::Internal)?;
        if parent.is_none() {
            return Err(ApiError::NotFound("parent category not found".to_string()));
        }
    }

    if name.trim().is_empty() {
        return Err(ApiError::BadRequest("name must not be empty".to_string()));
    }

    if existing.name() == name && existing.parent_id() == parent_id {
        return Ok(Json(category_to_response(&existing)));
    }

    let updated = Category::new(
        existing.id(),
        parent_id,
        name,
        existing.created_at(),
        chrono::Utc::now().timestamp(),
        existing.deleted_at(),
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    db::category::update(&state.pool, &updated)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok(Json(category_to_response(&updated)))
}

/// DELETE /api/v1/categories/:id — soft delete, or hard with ?force=true.
pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<DeleteCategoryQuery>,
) -> Result<StatusCode, ApiError> {
    let _ = db::category::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?
        .ok_or_else(|| ApiError::NotFound("category not found".to_string()))?;

    if q.force {
        db::category::hard_delete(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?;
    } else {
        db::category::soft_delete(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Deserialize "true" / "false" for force query param.
pub fn parse_force<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
    Ok(s.to_lowercase() == "true" || s == "1")
}

/// Router for /api/v1/categories (all five handlers).
pub fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/categories",
            get(list_categories).post(create_category),
        )
        .route(
            "/api/v1/categories/{id}",
            get(get_category)
                .patch(update_category)
                .delete(delete_category),
        )
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;
    use crate::config::Config;
    use crate::db;

    async fn test_pool() -> (AppState, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("category_test.db");
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
                    .join("pocketratings-category-test.pid")
                    .to_string_lossy()
                    .into_owned(),
            },
            pool,
        };
        (state, dir)
    }

    #[tokio::test]
    async fn list_categories_returns_empty_array_when_none() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/categories")
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
    async fn create_category_returns_201_and_body() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({ "name": "Groceries" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
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
        assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("Groceries"));
        let id_str = json.get("id").and_then(|v| v.as_str()).expect("id");
        assert!(Uuid::parse_str(id_str).is_ok());
        assert!(json.get("parent_id").is_none_or(serde_json::Value::is_null));
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
        // Persistence: entity exists in DB with same data
        let id = Uuid::parse_str(id_str).expect("uuid");
        let persisted = db::category::get_by_id(&state.pool, id).await.expect("db");
        let persisted = persisted.expect("category in db");
        assert_eq!(persisted.name(), "Groceries");
        assert_eq!(persisted.id(), id);
        assert_eq!(persisted.parent_id(), None);
        assert_eq!(persisted.deleted_at(), None);
    }

    #[tokio::test]
    async fn create_category_returns_400_when_name_empty() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let body = serde_json::json!({ "name": "" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_category_returns_404_when_not_found() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/categories/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_category_returns_200_when_found() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let body = serde_json::json!({ "name": "Electronics" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
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
                    .uri(format!("/api/v1/categories/{id}"))
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
            json.get("name").and_then(|v| v.as_str()),
            Some("Electronics")
        );
        assert!(json.get("parent_id").is_none_or(serde_json::Value::is_null));
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
    }

    #[tokio::test]
    async fn update_category_returns_200_when_no_change_does_not_persist() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let body = serde_json::json!({ "name": "Same" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
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
        let patch_body = serde_json::json!({ "name": "Same" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/categories/{id}"))
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
        assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("Same"));
    }

    #[tokio::test]
    async fn update_category_persists_new_values() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({ "name": "Original" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
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
        let patch_body = serde_json::json!({ "name": "UpdatedName" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/categories/{id}"))
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
            json.get("name").and_then(|v| v.as_str()),
            Some("UpdatedName")
        );
        let uuid = Uuid::parse_str(id).expect("uuid");
        let persisted = db::category::get_by_id(&state.pool, uuid)
            .await
            .expect("db");
        let persisted = persisted.expect("category in db");
        assert_eq!(persisted.name(), "UpdatedName");
    }

    #[tokio::test]
    async fn update_category_returns_404_when_category_missing() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let id = Uuid::new_v4();
        let patch_body = serde_json::json!({ "name": "New" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/categories/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_category_returns_204_soft_delete() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({ "name": "ToDelete" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
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
                    .uri(format!("/api/v1/categories/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        // Soft delete: row still exists with deleted_at set; normal get excludes it
        let active = db::category::get_by_id(&state.pool, uuid)
            .await
            .expect("db");
        assert!(
            active.is_none(),
            "get_by_id must exclude soft-deleted category"
        );
        let with_deleted = db::category::get_all(&state.pool, true).await.expect("db");
        let soft_deleted = with_deleted
            .iter()
            .find(|c| c.id() == uuid)
            .expect("row still in db");
        assert!(
            soft_deleted.deleted_at().is_some(),
            "deleted_at must be set after soft delete"
        );
    }

    #[tokio::test]
    async fn delete_category_hard_remove_row() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({ "name": "ToHardDelete" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
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
                    .uri(format!("/api/v1/categories/{id}?force=true"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let active = db::category::get_by_id(&state.pool, uuid)
            .await
            .expect("db");
        assert!(active.is_none());
        let with_deleted = db::category::get_all(&state.pool, true).await.expect("db");
        assert!(
            !with_deleted.iter().any(|c| c.id() == uuid),
            "hard delete must remove row from database"
        );
    }

    #[tokio::test]
    async fn delete_category_returns_404_when_not_found() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/categories/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_categories_with_parent_id_returns_children() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let parent_body = serde_json::json!({ "name": "Parent" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&parent_body).expect("json")))
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
        let parent: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        let parent_id = parent.get("id").and_then(|v| v.as_str()).expect("id");
        let child_body = serde_json::json!({ "name": "Child", "parent_id": parent_id });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&child_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/categories?parent_id={parent_id}"))
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
        assert_eq!(arr[0].get("name").and_then(|v| v.as_str()), Some("Child"));
        assert_eq!(
            arr[0]
                .get("children")
                .and_then(|v| v.as_array())
                .map(Vec::len),
            Some(0)
        );
    }

    #[tokio::test]
    async fn list_categories_with_depth_1_returns_only_roots() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({ "name": "Root1" })).expect("json"),
                    ))
                    .expect("request"),
            )
            .await
            .expect("service");
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({ "name": "Root2" })).expect("json"),
                    ))
                    .expect("request"),
            )
            .await
            .expect("service");
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/categories?depth=1")
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
        assert_eq!(arr.len(), 2, "depth=1 with no parent_id returns only roots");
        assert!(
            arr.iter()
                .any(|v| v.get("name").and_then(|x| x.as_str()) == Some("Root1"))
        );
        assert!(
            arr.iter()
                .any(|v| v.get("name").and_then(|x| x.as_str()) == Some("Root2"))
        );
        for item in arr {
            assert!(
                item.get("children")
                    .and_then(|v| v.as_array())
                    .is_some_and(Vec::is_empty)
            );
        }
    }

    #[tokio::test]
    async fn create_category_returns_404_when_parent_missing() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let fake_parent_id = Uuid::new_v4();
        let body = serde_json::json!({ "name": "Child", "parent_id": fake_parent_id.to_string() });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn create_category_returns_400_when_duplicate_name_under_same_parent() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let body = serde_json::json!({ "name": "Dupe" });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/categories")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
