//! Locations REST API: list, get, create, update, delete.

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
use crate::domain::location::Location;

/// Minimal location info for embedding in purchase (and future) responses.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LocationRef {
    pub id: Uuid,
    pub name: String,
}

/// Request body for creating a location.
#[derive(Debug, Deserialize)]
pub struct CreateLocationRequest {
    pub name: String,
}

/// Request body for partial update.
#[derive(Debug, Deserialize)]
pub struct UpdateLocationRequest {
    pub name: Option<String>,
}

/// Query params for delete (optional force).
#[derive(Debug, Default, Deserialize)]
pub struct DeleteLocationQuery {
    #[serde(default, deserialize_with = "parse_force")]
    pub force: bool,
}

/// Response body: location with `deleted_at` as i64.
#[derive(Debug, serde::Serialize)]
pub struct LocationResponse {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

fn location_to_response(l: &Location) -> LocationResponse {
    LocationResponse {
        id: l.id(),
        name: l.name().to_string(),
        deleted_at: l.deleted_at(),
    }
}

/// Map `DbError` to `ApiError` for location operations.
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

/// GET /api/v1/locations — list all active locations.
pub async fn list_locations(
    State(state): State<AppState>,
) -> Result<Json<Vec<LocationResponse>>, ApiError> {
    let list = db::location::get_all(&state.pool)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok(Json(list.iter().map(location_to_response).collect()))
}

/// GET /api/v1/locations/:id — get one location.
pub async fn get_location(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LocationResponse>, ApiError> {
    let location = db::location::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let location = location.ok_or_else(|| ApiError::NotFound("location not found".to_string()))?;
    Ok(Json(location_to_response(&location)))
}

/// POST /api/v1/locations — create a location.
pub async fn create_location(
    State(state): State<AppState>,
    Json(body): Json<CreateLocationRequest>,
) -> Result<(StatusCode, Json<LocationResponse>), ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name must not be empty".to_string()));
    }
    let id = Uuid::new_v4();
    let location = Location::new(id, body.name.trim().to_string(), None)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    db::location::insert(&state.pool, &location)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok((StatusCode::CREATED, Json(location_to_response(&location))))
}

/// PATCH /api/v1/locations/:id — partial update; only persist if name changed.
pub async fn update_location(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateLocationRequest>,
) -> Result<Json<LocationResponse>, ApiError> {
    let existing = db::location::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?;
    let existing = existing.ok_or_else(|| ApiError::NotFound("location not found".to_string()))?;

    let new_name = body
        .name
        .as_deref()
        .map(str::trim)
        .map(std::string::ToString::to_string);
    let name = new_name.unwrap_or_else(|| existing.name().to_string());

    if name.trim().is_empty() {
        return Err(ApiError::BadRequest("name must not be empty".to_string()));
    }

    if existing.name() == name {
        return Ok(Json(location_to_response(&existing)));
    }

    let updated = Location::new(existing.id(), name, existing.deleted_at())
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    db::location::update(&state.pool, &updated)
        .await
        .map_err(|e| map_db_error(&e))?;
    Ok(Json(location_to_response(&updated)))
}

/// DELETE /api/v1/locations/:id — soft delete, or hard with ?force=true.
pub async fn delete_location(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<DeleteLocationQuery>,
) -> Result<StatusCode, ApiError> {
    let _ = db::location::get_by_id(&state.pool, id)
        .await
        .map_err(|e| map_db_error(&e))?
        .ok_or_else(|| ApiError::NotFound("location not found".to_string()))?;

    if q.force {
        db::location::hard_delete(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?;
    } else {
        db::location::soft_delete(&state.pool, id)
            .await
            .map_err(|e| map_db_error(&e))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Router for /api/v1/locations (all five handlers).
pub fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/locations",
            get(list_locations).post(create_location),
        )
        .route(
            "/api/v1/locations/{id}",
            get(get_location)
                .patch(update_location)
                .delete(delete_location),
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

    async fn test_pool() -> (AppState, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("location_test.db");
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
                    .join("pocketratings-location-test.pid")
                    .to_string_lossy()
                    .into_owned(),
            },
            pool,
        };
        (state, dir)
    }

    #[tokio::test]
    async fn list_locations_returns_empty_array_when_none() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/locations")
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
    async fn create_location_returns_201_and_body() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({ "name": "Supermarket" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/locations")
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
            json.get("name").and_then(|v| v.as_str()),
            Some("Supermarket")
        );
        let id_str = json.get("id").and_then(|v| v.as_str()).expect("id");
        assert!(Uuid::parse_str(id_str).is_ok());
        assert!(
            json.get("deleted_at")
                .is_none_or(serde_json::Value::is_null)
        );
        let id = Uuid::parse_str(id_str).expect("uuid");
        let persisted = db::location::get_by_id(&state.pool, id).await.expect("db");
        let persisted = persisted.expect("location in db");
        assert_eq!(persisted.name(), "Supermarket");
        assert_eq!(persisted.id(), id);
        assert_eq!(persisted.deleted_at(), None);
    }

    #[tokio::test]
    async fn create_location_returns_400_when_name_empty() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let body = serde_json::json!({ "name": "" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/locations")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_location_returns_404_when_not_found() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/locations/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_location_returns_200_when_found() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let body = serde_json::json!({ "name": "Corner store" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/locations")
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
                    .uri(format!("/api/v1/locations/{id}"))
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
            Some("Corner store")
        );
        assert!(
            json.get("deleted_at")
                .is_none_or(serde_json::Value::is_null)
        );
    }

    #[tokio::test]
    async fn update_location_returns_200_when_no_change_does_not_persist() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let body = serde_json::json!({ "name": "Same" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/locations")
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
                    .uri(format!("/api/v1/locations/{id}"))
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
    async fn update_location_persists_new_values() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({ "name": "Original" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/locations")
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
                    .uri(format!("/api/v1/locations/{id}"))
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
        let persisted = db::location::get_by_id(&state.pool, uuid)
            .await
            .expect("db");
        let persisted = persisted.expect("location in db");
        assert_eq!(persisted.name(), "UpdatedName");
    }

    #[tokio::test]
    async fn update_location_returns_404_when_missing() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let id = Uuid::new_v4();
        let patch_body = serde_json::json!({ "name": "New" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/locations/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn update_location_returns_400_when_name_empty() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let body = serde_json::json!({ "name": "Valid" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/locations")
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
        let patch_body = serde_json::json!({ "name": "" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/locations/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_location_returns_204_soft_delete() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({ "name": "ToDelete" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/locations")
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
                    .uri(format!("/api/v1/locations/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let active = db::location::get_by_id(&state.pool, uuid)
            .await
            .expect("db");
        assert!(
            active.is_none(),
            "get_by_id must exclude soft-deleted location"
        );
        let with_deleted = db::location::get_all_with_deleted(&state.pool)
            .await
            .expect("db");
        let soft_deleted = with_deleted
            .iter()
            .find(|l| l.id() == uuid)
            .expect("row still in db");
        assert!(
            soft_deleted.deleted_at().is_some(),
            "deleted_at must be set after soft delete"
        );
    }

    #[tokio::test]
    async fn delete_location_hard_remove_row() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state.clone());
        let body = serde_json::json!({ "name": "ToHardDelete" });
        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/locations")
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
                    .uri(format!("/api/v1/locations/{id}?force=true"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let active = db::location::get_by_id(&state.pool, uuid)
            .await
            .expect("db");
        assert!(active.is_none());
        let with_deleted = db::location::get_all_with_deleted(&state.pool)
            .await
            .expect("db");
        assert!(
            !with_deleted.iter().any(|l| l.id() == uuid),
            "hard delete must remove row from database"
        );
    }

    #[tokio::test]
    async fn delete_location_returns_404_when_not_found() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/locations/{id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_locations_returns_populated_list() {
        let (state, _dir) = test_pool().await;
        let app = route().with_state(state);
        let body = serde_json::json!({ "name": "A" });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/locations")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/locations")
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
        assert_eq!(arr[0].get("name").and_then(|v| v.as_str()), Some("A"));
    }
}
