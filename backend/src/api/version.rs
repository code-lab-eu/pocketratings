//! `GET /api/v1/version` — returns the current crate version.

use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use super::state::AppState;

/// Response body for `GET /api/v1/version`.
#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub version: String,
}

/// Handler for `GET /api/v1/version`. Returns the current crate version.
pub async fn version(State(_): State<AppState>) -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Route for this endpoint (for use in the API router).
pub fn route() -> Router<AppState> {
    Router::new().route("/api/v1/version", get(version))
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;

    use crate::api::state::AppState;
    use crate::config::Config;

    use super::route;

    fn test_state() -> AppState {
        AppState {
            config: Config {
                database_path: ":memory:".to_string(),
                jwt_secret: "test".to_string(),
                jwt_expiration_seconds: 3600,
                jwt_refresh_threshold_seconds: 600,
                bind: "127.0.0.1:3099".to_string(),
                pid_file: std::env::temp_dir()
                    .join("pocketratings-test.pid")
                    .to_string_lossy()
                    .into_owned(),
            },
            pool: SqlitePool::connect_lazy("sqlite::memory:").expect("in-memory pool"),
        }
    }

    #[tokio::test]
    async fn get_version_returns_200_and_json_with_version() {
        use axum::body::Body;
        use axum::http::Request;
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let app = route().with_state(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/version")
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("service call");

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body collect")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).expect("response body is JSON");
        let version = json
            .get("version")
            .and_then(|v| v.as_str())
            .expect("response has version string");
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }
}
