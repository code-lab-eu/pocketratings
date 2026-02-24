//! User-related API types (e.g. refs for embedding in other responses).

use serde::Serialize;
use uuid::Uuid;

/// Minimal user info for embedding in purchase (and future) responses.
#[derive(Debug, Clone, Serialize)]
pub struct UserRef {
    pub id: Uuid,
    pub name: String,
}
