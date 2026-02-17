//! Integration tests for REST auth: login and protected route (GET /api/v1/me).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use pocketratings::api::{AppState, router};
use pocketratings::auth::password;
use pocketratings::config::Config;
use pocketratings::db;
use tower::ServiceExt;
use uuid::Uuid;

async fn test_pool_with_user(
    email: &str,
    password_plain: &str,
) -> (AppState, Uuid, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("rest_auth.db");
    let path_str = db_path.to_str().expect("path utf-8").to_string();
    let pool = db::create_pool(&path_str).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrate");
    let id = Uuid::new_v4();
    let hash = password::hash_password(password_plain).expect("hash");
    let now = 1_700_000_000i64;
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind("Alice")
    .bind(email)
    .bind(&hash)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert user");
    let config = Config {
        database_path: path_str.clone(),
        jwt_secret: "test-secret".to_string(),
        jwt_expiration_seconds: 3600,
        jwt_refresh_threshold_seconds: 600,
        bind: "127.0.0.1:3099".to_string(),
        pid_file: std::env::temp_dir()
            .join("pocketratings-rest-auth-test.pid")
            .to_string_lossy()
            .into_owned(),
    };
    let state = AppState { config, pool };
    (state, id, dir)
}

#[tokio::test]
async fn login_then_get_me_returns_current_user_id() {
    let (state, user_id, _dir) = test_pool_with_user("alice@example.com", "secret123").await;
    let login_body = serde_json::json!({
        "email": "alice@example.com",
        "password": "secret123"
    });

    let app_login = router(state.clone());
    let login_resp = app_login
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
        .expect("response has token");

    let app_me = router(state);
    let me_resp = app_me
        .oneshot(
            Request::builder()
                .uri("/api/v1/me")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("service");

    assert_eq!(me_resp.status(), StatusCode::OK);
    let me_bytes = me_resp
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let me_json: serde_json::Value = serde_json::from_slice(&me_bytes).expect("json");
    assert_eq!(
        me_json.get("user_id").and_then(|v| v.as_str()),
        Some(user_id.to_string().as_str())
    );
    assert_eq!(me_json.get("name").and_then(|v| v.as_str()), Some("Alice"));
}
