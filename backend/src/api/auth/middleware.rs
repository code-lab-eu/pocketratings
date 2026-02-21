//! Auth middleware: require valid JWT and attach current user id to request.

use axum::{
    extract::Request,
    http::{HeaderValue, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::api::auth::jwt;
use crate::api::error::ApiError;
use crate::api::state::AppState;
use axum::extract::State;

/// Response header with a new JWT when sliding refresh is applied.
pub const X_NEW_TOKEN: &str = "x-new-token";

/// Extension value: the current authenticated user's id (set by auth middleware).
#[derive(Debug, Clone)]
pub struct CurrentUserId(pub Uuid);

/// Auth middleware: require `Authorization: Bearer <token>`, verify JWT, set `CurrentUserId` in extensions.
/// If the token expires within `jwt_refresh_threshold_seconds`, issues a new token and adds `X-New-Token` header.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(auth_header) = request.headers().get(header::AUTHORIZATION) else {
        return ApiError::Unauthorized("missing authorization header".to_string()).into_response();
    };
    let Ok(auth_str) = auth_header.to_str() else {
        return ApiError::Unauthorized("invalid authorization header".to_string()).into_response();
    };
    let token = auth_str.strip_prefix("Bearer ").unwrap_or(auth_str);
    if token.is_empty() || token == auth_str {
        return ApiError::Unauthorized("missing or invalid bearer token".to_string())
            .into_response();
    }
    let Ok(claims) = jwt::verify_token(&state.config.jwt_secret, token) else {
        return ApiError::Unauthorized("invalid or expired token".to_string()).into_response();
    };
    let Ok(user_id) = Uuid::parse_str(&claims.sub) else {
        return ApiError::Unauthorized("invalid token subject".to_string()).into_response();
    };
    request.extensions_mut().insert(CurrentUserId(user_id));

    let mut response = next.run(request).await;

    // Sliding expiration: if token expires within threshold, issue new token and set X-New-Token header.
    let now = jsonwebtoken::get_current_timestamp();
    let refresh_at = now + state.config.jwt_refresh_threshold_seconds;
    if claims.exp <= refresh_at
        && let Ok(new_token) = jwt::issue_token(
            &state.config.jwt_secret,
            user_id,
            state.config.jwt_expiration_seconds,
        )
        && let Ok(hv) = HeaderValue::from_str(&new_token)
    {
        response.headers_mut().insert(X_NEW_TOKEN, hv);
    }

    response
}

/// Response body for `GET /api/v1/me`.
#[derive(Debug, serde::Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub name: String,
}

/// `GET /api/v1/me` — return current user id and name. Protected.
pub async fn me(
    axum::extract::Extension(CurrentUserId(user_id)): axum::extract::Extension<CurrentUserId>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<axum::Json<MeResponse>, ApiError> {
    let user = crate::db::user::get_by_id(&state.pool, user_id)
        .await
        .map_err(|_| ApiError::Internal)?;
    let user = user.ok_or_else(|| ApiError::NotFound("user not found".to_string()))?;
    Ok(axum::Json(MeResponse {
        user_id: user_id.to_string(),
        name: user.name().to_string(),
    }))
}

/// Router for the /me endpoint (no layer; layer is applied in the main router).
pub fn me_route() -> axum::Router<AppState> {
    axum::Router::new().route("/api/v1/me", axum::routing::get(me))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::api::auth::jwt;
    use crate::api::router;
    use crate::api::state::AppState;
    use crate::auth::password;
    use crate::config::Config;
    use crate::db;
    use uuid::Uuid;

    fn test_config(db_path: &str) -> Config {
        Config {
            database_path: db_path.to_string(),
            jwt_secret: "test-secret".to_string(),
            jwt_expiration_seconds: 3600,
            jwt_refresh_threshold_seconds: 600,
            bind: "127.0.0.1:3099".to_string(),
            pid_file: std::env::temp_dir()
                .join("pocketratings-me-test.pid")
                .to_string_lossy()
                .into_owned(),
        }
    }

    async fn test_pool_with_user(
        email: &str,
        password_plain: &str,
    ) -> (AppState, Uuid, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("me_test.db");
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
        .bind("Test User")
        .bind(email)
        .bind(&hash)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert user");
        let state = AppState {
            config: test_config(&path_str),
            pool,
        };
        (state, id, dir)
    }

    #[tokio::test]
    async fn get_me_returns_401_without_auth_header() {
        let (state, _, _dir) = test_pool_with_user("u@ex.co", "pass").await;
        let app = router::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/me")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("service");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn get_me_returns_200_and_user_id_and_name_with_valid_token() {
        let (state, user_id, _dir) = test_pool_with_user("u@ex.co", "pass").await;
        let token = jwt::issue_token(
            &state.config.jwt_secret,
            user_id,
            state.config.jwt_expiration_seconds,
        )
        .expect("issue token");
        let app = router::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {token}"))
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
        assert_eq!(
            json.get("user_id").and_then(|v| v.as_str()),
            Some(user_id.to_string().as_str())
        );
        assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("Test User"));
    }
}
