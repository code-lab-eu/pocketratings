//! Axum router — composes route modules into the API router.

use axum::{Router, middleware};

use super::auth::{auth_middleware, login_route, me_route};
use super::state::AppState;

/// Build the API router with all v1 routes.
pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .merge(super::version::route())
        .merge(login_route());

    let protected = me_route().route_layer(middleware::from_fn_with_state(
        state.clone(),
        auth_middleware,
    ));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
}
