//! REST routes and handlers.

mod auth;
mod category;
mod error;
mod location;
mod product;
mod purchase;
mod review;
mod router;
mod server;
mod state;
mod user;
mod version;

pub use error::{ApiError, ErrorBody};
pub use router::router;
pub use server::{ServerError, start as server_start};
pub use state::AppState;
pub use version::{VersionResponse, version};
