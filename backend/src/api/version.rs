//! `GET /api/v1/version` — returns the current crate version.

use axum::{Json, Router, routing::get};
use serde::Serialize;

/// Response body for `GET /api/v1/version`.
#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub version: String,
}

/// Handler for `GET /api/v1/version`. Returns the current crate version.
pub async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Route for this endpoint (for use in the API router).
pub fn route() -> Router {
    Router::new().route("/api/v1/version", get(version))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::route;

    #[tokio::test]
    async fn get_version_returns_200_and_json_with_version() {
        let app = route();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/version")
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("service call");

        assert_eq!(response.status(), StatusCode::OK);

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
