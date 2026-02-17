//! REST routes and handlers.

mod router;
mod server;
mod version;

pub use router::router;
pub use server::{ServerError, start as server_start};
pub use version::{VersionResponse, version};
