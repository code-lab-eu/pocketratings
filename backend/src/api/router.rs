//! Axum router — composes route modules into the API router.

use axum::Router;

/// Build the API router with all v1 routes.
pub fn router() -> Router {
    Router::new().merge(super::version::route())
}
