//! `POST /api/v1/auth/login` — authenticate and receive a JWT.

use axum::{Json, extract::State};
use serde::Deserialize;

use crate::api::{error::ApiError, state::AppState};
use crate::auth::password;
use crate::db;

/// Request body for login.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Response body: token only.
#[derive(Debug, serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
}

/// Handler for `POST /api/v1/auth/login`.
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    if body.email.is_empty() {
        return Err(ApiError::BadRequest("email is required".to_string()));
    }
    let user = db::user::get_by_email(&state.pool, &body.email)
        .await
        .map_err(|_| ApiError::Internal)?;
    let user =
        user.ok_or_else(|| ApiError::Unauthorized("invalid email or password".to_string()))?;
    let ok = password::verify_password(&body.password, user.password())
        .map_err(|_| ApiError::Internal)?;
    if !ok {
        return Err(ApiError::Unauthorized(
            "invalid email or password".to_string(),
        ));
    }
    let token = crate::api::auth::jwt::issue_token(
        &state.config.jwt_secret,
        user.id(),
        state.config.jwt_expiration_seconds,
    )
    .map_err(|_| ApiError::Internal)?;
    Ok(Json(LoginResponse { token }))
}

/// Route for this endpoint (public, no auth).
pub fn route() -> axum::Router<crate::api::state::AppState> {
    axum::Router::new().route("/api/v1/auth/login", axum::routing::post(login))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::api::auth::login::route;
    use crate::api::state::AppState;
    use crate::auth::password;
    use crate::config::Config;
    use crate::db;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    fn test_config(db_path: &str) -> Config {
        Config {
            database_path: db_path.to_string(),
            jwt_secret: "test-secret".to_string(),
            jwt_expiration_seconds: 3600,
            jwt_refresh_threshold_seconds: 600,
            bind: "127.0.0.1:3099".to_string(),
            pid_file: std::env::temp_dir()
                .join("pocketratings-login-test.pid")
                .to_string_lossy()
                .into_owned(),
        }
    }

    /// Create a pool backed by a temp file (so migrations and connections share the same DB).
    /// Returns (pool, `db_path_string`); keep _dir alive for the test so the file is not removed.
    async fn test_pool() -> (SqlitePool, String, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("login_test.db");
        let path_str = db_path.to_str().expect("path utf-8").to_string();
        let pool = db::create_pool(&path_str).await.expect("pool");
        db::run_migrations(&pool).await.expect("migrate");
        (pool, path_str, dir)
    }

    async fn setup_db_with_user(pool: &SqlitePool, email: &str, password_plain: &str) -> Uuid {
        let id = Uuid::new_v4();
        let name = "Test User";
        let hash = password::hash_password(password_plain).expect("hash");
        let now = 1_700_000_000i64;
        sqlx::query(
            "INSERT INTO users (id, name, email, password, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(name)
        .bind(email)
        .bind(&hash)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .expect("insert user");
        id
    }

    #[tokio::test]
    async fn login_returns_200_and_token_when_credentials_valid() {
        let (pool, path_str, _dir) = test_pool().await;
        let _ = setup_db_with_user(&pool, "u@example.com", "secret123").await;
        let state = AppState {
            config: test_config(&path_str),
            pool,
        };
        let app = route().with_state(state);

        let body = serde_json::json!({ "email": "u@example.com", "password": "secret123" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
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
        assert!(json.get("token").and_then(|t| t.as_str()).is_some());
    }

    #[tokio::test]
    async fn login_returns_401_when_password_wrong() {
        let (pool, path_str, _dir) = test_pool().await;
        let _ = setup_db_with_user(&pool, "u@example.com", "correct").await;
        let state = AppState {
            config: test_config(&path_str),
            pool,
        };
        let app = route().with_state(state);

        let body = serde_json::json!({ "email": "u@example.com", "password": "wrong" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn login_returns_401_when_user_missing() {
        let (pool, path_str, _dir) = test_pool().await;
        let state = AppState {
            config: test_config(&path_str),
            pool,
        };
        let app = route().with_state(state);

        let body = serde_json::json!({ "email": "nobody@example.com", "password": "any" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
