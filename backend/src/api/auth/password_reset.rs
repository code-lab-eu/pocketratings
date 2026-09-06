//! `POST /api/v1/auth/password-reset` and `.../validate` — set a password with a reset link.
//!
//! Both endpoints are public: the token is the credential. It travels in the request body rather
//! than the path so it stays out of access logs. Invalid, expired, and already used tokens all
//! answer with the same 401, so the three cases cannot be told apart.

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use serde::Deserialize;

use crate::api::{error::ApiError, state::AppState};
use crate::auth::{password, reset_token};
use crate::db;

/// Request body for validating a reset token.
#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub token: String,
}

/// Request body for setting a new password with a reset token.
#[derive(Debug, Deserialize)]
pub struct ResetRequest {
    pub token: String,
    pub password: String,
}

/// The one answer given for any token that cannot be used.
fn invalid_link() -> ApiError {
    ApiError::Unauthorized("Invalid or expired reset link.".to_string())
}

/// Handler for `POST /api/v1/auth/password-reset/validate`.
pub async fn validate(
    State(state): State<AppState>,
    Json(body): Json<ValidateRequest>,
) -> Result<StatusCode, ApiError> {
    let usable = db::password_reset::is_valid(
        &state.pool,
        &reset_token::hash(&body.token),
        Utc::now().timestamp(),
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    if usable {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(invalid_link())
    }
}

/// Handler for `POST /api/v1/auth/password-reset`.
pub async fn reset(
    State(state): State<AppState>,
    Json(body): Json<ResetRequest>,
) -> Result<StatusCode, ApiError> {
    if body.password.is_empty() {
        return Err(ApiError::BadRequest("Password is required.".to_string()));
    }
    let password_hash = password::hash_password(&body.password).map_err(|_| ApiError::Internal)?;

    let consumed = db::password_reset::consume_and_set_password(
        &state.pool,
        &reset_token::hash(&body.token),
        &password_hash,
        Utc::now().timestamp(),
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    if consumed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(invalid_link())
    }
}

/// Routes for this endpoint (public, no auth).
pub fn route() -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            "/api/v1/auth/password-reset/validate",
            axum::routing::post(validate),
        )
        .route("/api/v1/auth/password-reset", axum::routing::post(reset))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    use super::*;
    use crate::auth::reset_token::TOKEN_TTL_SECONDS;
    use crate::test_helpers::{api_test_state, insert_user};
    use http_body_util::BodyExt;
    use uuid::Uuid;

    /// Send a POST to one of the reset endpoints and return the response status and JSON body.
    async fn post(
        state: AppState,
        path: &str,
        body: &serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let response = route()
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(body).expect("json")))
                    .expect("request"),
            )
            .await
            .expect("service");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json = if bytes.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&bytes).expect("json body")
        };
        (status, json)
    }

    /// Store a token for `user_id` that was created `age_seconds` ago, and return the raw token.
    async fn create_token(state: &AppState, user_id: Uuid, age_seconds: i64) -> String {
        let token = reset_token::generate();
        db::password_reset::create(
            &state.pool,
            user_id,
            &reset_token::hash(&token),
            Utc::now().timestamp() - age_seconds,
        )
        .await
        .expect("create token");
        token
    }

    /// Read the stored password hash of a user.
    async fn stored_password(state: &AppState, user_id: Uuid) -> String {
        db::user::get_by_id(&state.pool, user_id, false)
            .await
            .expect("get user")
            .expect("user exists")
            .password()
            .to_string()
    }

    fn assert_invalid_link(status: StatusCode, body: &serde_json::Value) {
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"], "unauthorized");
        assert_eq!(body["message"], "Invalid or expired reset link.");
    }

    #[tokio::test]
    async fn validate_returns_204_for_a_fresh_token() {
        let (state, _dir) = api_test_state().await;
        let user_id = insert_user(&state.pool, "Alice", "alice@example.com").await;
        let token = create_token(&state, user_id, 0).await;

        let (status, _body) = post(
            state,
            "/api/v1/auth/password-reset/validate",
            &serde_json::json!({ "token": token }),
        )
        .await;

        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn validate_returns_401_for_an_unknown_token() {
        let (state, _dir) = api_test_state().await;

        let (status, body) = post(
            state,
            "/api/v1/auth/password-reset/validate",
            &serde_json::json!({ "token": "not-a-real-token" }),
        )
        .await;

        assert_invalid_link(status, &body);
    }

    #[tokio::test]
    async fn validate_returns_401_for_an_expired_token() {
        let (state, _dir) = api_test_state().await;
        let user_id = insert_user(&state.pool, "Alice", "alice@example.com").await;
        let token = create_token(&state, user_id, TOKEN_TTL_SECONDS + 60).await;

        let (status, body) = post(
            state,
            "/api/v1/auth/password-reset/validate",
            &serde_json::json!({ "token": token }),
        )
        .await;

        assert_invalid_link(status, &body);
    }

    #[tokio::test]
    async fn validate_returns_401_for_an_already_used_token() {
        let (state, _dir) = api_test_state().await;
        let user_id = insert_user(&state.pool, "Alice", "alice@example.com").await;
        let token = create_token(&state, user_id, 0).await;
        post(
            state.clone(),
            "/api/v1/auth/password-reset",
            &serde_json::json!({ "token": token, "password": "newpass" }),
        )
        .await;

        let (status, body) = post(
            state,
            "/api/v1/auth/password-reset/validate",
            &serde_json::json!({ "token": token }),
        )
        .await;

        assert_invalid_link(status, &body);
    }

    #[tokio::test]
    async fn reset_returns_204_and_sets_the_new_password() {
        let (state, _dir) = api_test_state().await;
        let user_id = insert_user(&state.pool, "Alice", "alice@example.com").await;
        let token = create_token(&state, user_id, 0).await;

        let (status, _body) = post(
            state.clone(),
            "/api/v1/auth/password-reset",
            &serde_json::json!({ "token": token, "password": "newpass" }),
        )
        .await;

        assert_eq!(status, StatusCode::NO_CONTENT);
        let hash = stored_password(&state, user_id).await;
        assert!(password::verify_password("newpass", &hash).expect("verify new"));
    }

    #[tokio::test]
    async fn reusing_a_token_returns_401_and_leaves_the_password_unchanged() {
        let (state, _dir) = api_test_state().await;
        let user_id = insert_user(&state.pool, "Alice", "alice@example.com").await;
        let token = create_token(&state, user_id, 0).await;
        post(
            state.clone(),
            "/api/v1/auth/password-reset",
            &serde_json::json!({ "token": token, "password": "newpass" }),
        )
        .await;
        let after_first = stored_password(&state, user_id).await;

        let (status, body) = post(
            state.clone(),
            "/api/v1/auth/password-reset",
            &serde_json::json!({ "token": token, "password": "secondpass" }),
        )
        .await;

        assert_invalid_link(status, &body);
        assert_eq!(stored_password(&state, user_id).await, after_first);
    }

    #[tokio::test]
    async fn reset_with_an_expired_token_returns_401_and_leaves_the_password_unchanged() {
        let (state, _dir) = api_test_state().await;
        let user_id = insert_user(&state.pool, "Alice", "alice@example.com").await;
        let token = create_token(&state, user_id, TOKEN_TTL_SECONDS + 60).await;
        let before = stored_password(&state, user_id).await;

        let (status, body) = post(
            state.clone(),
            "/api/v1/auth/password-reset",
            &serde_json::json!({ "token": token, "password": "newpass" }),
        )
        .await;

        assert_invalid_link(status, &body);
        assert_eq!(stored_password(&state, user_id).await, before);
    }

    #[tokio::test]
    async fn reset_with_an_empty_password_returns_400() {
        let (state, _dir) = api_test_state().await;
        let user_id = insert_user(&state.pool, "Alice", "alice@example.com").await;
        let token = create_token(&state, user_id, 0).await;
        let before = stored_password(&state, user_id).await;

        let (status, body) = post(
            state.clone(),
            "/api/v1/auth/password-reset",
            &serde_json::json!({ "token": token, "password": "" }),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "bad_request");
        assert_eq!(body["message"], "Password is required.");
        assert_eq!(stored_password(&state, user_id).await, before);
    }
}
