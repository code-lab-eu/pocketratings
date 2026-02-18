//! Integration test: all protected routes return 403 when the request has no valid auth.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use pocketratings::api::{AppState, router};
use pocketratings::auth::password;
use pocketratings::config::Config;
use pocketratings::db;
use tower::ServiceExt;
use uuid::Uuid;

/// Placeholder UUID for path params (resource need not exist; we only assert 403).
const PLACEHOLDER_ID: &str = "00000000-0000-0000-0000-000000000001";

/// Protected routes: (method, path). Paths with `:id` use PLACEHOLDER_ID.
fn protected_routes() -> Vec<(Method, String)> {
    let id_path_cat = format!("/api/v1/categories/{}", PLACEHOLDER_ID);
    let id_path_loc = format!("/api/v1/locations/{}", PLACEHOLDER_ID);
    let id_path_prod = format!("/api/v1/products/{}", PLACEHOLDER_ID);
    vec![
        (Method::GET, "/api/v1/me".to_string()),
        (Method::GET, "/api/v1/categories".to_string()),
        (Method::GET, id_path_cat.clone()),
        (Method::POST, "/api/v1/categories".to_string()),
        (Method::PATCH, id_path_cat.clone()),
        (Method::DELETE, id_path_cat),
        (Method::GET, "/api/v1/locations".to_string()),
        (Method::GET, id_path_loc.clone()),
        (Method::POST, "/api/v1/locations".to_string()),
        (Method::PATCH, id_path_loc.clone()),
        (Method::DELETE, id_path_loc),
        (Method::GET, "/api/v1/products".to_string()),
        (Method::GET, id_path_prod.clone()),
        (Method::POST, "/api/v1/products".to_string()),
        (Method::PATCH, id_path_prod.clone()),
        (Method::DELETE, id_path_prod),
    ]
}

async fn test_state_with_user() -> (AppState, String, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("protected_403.db");
    let path_str = db_path.to_str().expect("path utf-8").to_string();
    let pool = db::create_pool(&path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrate");
    let hash = password::hash_password("testpass").expect("hash");
    let now = 1_700_000_000i64;
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind("Test User")
    .bind("test@example.com")
    .bind(&hash)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert user");
    let config = Config {
        database_path: path_str,
        jwt_secret: "test-secret".to_string(),
        jwt_expiration_seconds: 3600,
        jwt_refresh_threshold_seconds: 600,
        bind: "127.0.0.1:0".to_string(),
        pid_file: std::env::temp_dir()
            .join("pocketratings-protected-403-test.pid")
            .to_string_lossy()
            .into_owned(),
    };
    let state = AppState {
        config: config.clone(),
        pool,
    };
    let app = router(state.clone());
    let login_body = serde_json::json!({
        "email": "test@example.com",
        "password": "testpass"
    });
    let login_resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_body).expect("json")))
                .expect("request"),
        )
        .await
        .expect("service");
    assert_eq!(login_resp.status(), StatusCode::OK);
    let login_bytes = login_resp
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let login_json: serde_json::Value = serde_json::from_slice(&login_bytes).expect("json");
    let token = login_json
        .get("token")
        .and_then(|t| t.as_str())
        .expect("response has token")
        .to_string();
    (state, token, dir)
}

#[tokio::test]
async fn all_protected_routes_return_403_without_auth() {
    let (state, valid_token, _dir) = test_state_with_user().await;
    let app = router(state);

    for (method, path) in protected_routes() {
        let body_bytes = if method == Method::POST || method == Method::PATCH {
            r#"{"name":"x"}"#.as_bytes().to_vec()
        } else {
            Vec::new()
        };

        // Test 1: No authentication token => 403
        let mut req_builder = Request::builder().method(method.clone()).uri(path.clone());
        if method == Method::POST || method == Method::PATCH {
            req_builder = req_builder.header("content-type", "application/json");
        }
        let body = if body_bytes.is_empty() {
            Body::empty()
        } else {
            Body::from(body_bytes.clone())
        };
        let request = req_builder.body(body).expect("request");
        let response = app.clone().oneshot(request).await.expect("service");
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "{} {} should return 403 without auth header",
            method,
            path
        );

        // Test 2: Invalid authentication token => 403
        let mut req_builder = Request::builder().method(method.clone()).uri(path.clone());
        if method == Method::POST || method == Method::PATCH {
            req_builder = req_builder.header("content-type", "application/json");
        }
        let body = if body_bytes.is_empty() {
            Body::empty()
        } else {
            Body::from(body_bytes.clone())
        };
        let request = req_builder
            .header("authorization", "Bearer invalid-token-here")
            .body(body)
            .expect("request");
        let response = app.clone().oneshot(request).await.expect("service");
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "{} {} should return 403 with invalid token",
            method,
            path
        );

        // Test 3: Correct authentication token => !403
        let mut req_builder = Request::builder().method(method.clone()).uri(path.clone());
        if method == Method::POST || method == Method::PATCH {
            req_builder = req_builder.header("content-type", "application/json");
        }
        let body = if body_bytes.is_empty() {
            Body::empty()
        } else {
            Body::from(body_bytes)
        };
        let request = req_builder
            .header("authorization", format!("Bearer {}", valid_token))
            .body(body)
            .expect("request");
        let response = app.clone().oneshot(request).await.expect("service");
        assert_ne!(
            response.status(),
            StatusCode::FORBIDDEN,
            "{} {} should not return 403 with valid token (may return 200, 201, 204, 404, etc.)",
            method,
            path
        );
    }
}
