//! HTTP server startup and graceful shutdown.

use std::future::Future;
use std::pin::Pin;

use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tokio::signal;

use crate::api::router;
use crate::config::Config;

/// Wait for SIGTERM (Unix) or never complete (other platforms).
fn wait_for_sigterm() -> Pin<Box<dyn Future<Output = ()> + Send>> {
    #[cfg(unix)]
    {
        Box::pin(async {
            let mut sig = match signal::unix::signal(signal::unix::SignalKind::terminate()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("failed to install SIGTERM handler: {}", e);
                    return;
                }
            };
            sig.recv().await;
        })
    }
    #[cfg(not(unix))]
    Box::pin(std::future::pending())
}

/// Run the API server until shutdown (SIGINT or SIGTERM).
///
/// Binds to `bind_addr`, serves the router with graceful shutdown.
/// The database pool is available for future routes (e.g. auth); the version endpoint does not use it.
///
/// # Errors
///
/// Returns an error if binding to `bind_addr` fails or if the server exits with an error.
pub async fn start(
    _config: &Config,
    _pool: &SqlitePool,
    bind_addr: &str,
) -> Result<(), ServerError> {
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(ServerError::Bind)?;
    let addr = listener.local_addr().map_err(ServerError::Bind)?;
    tracing::info!("listening on {}", addr);

    let shutdown = async {
        tokio::select! {
            _ = signal::ctrl_c() => {}
            () = wait_for_sigterm() => {}
        }
    };

    axum::serve(listener, router::router())
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(ServerError::Serve)?;

    Ok(())
}

/// Errors that can occur when starting or running the server.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("failed to bind to address: {0}")]
    Bind(std::io::Error),

    #[error("server error: {0}")]
    Serve(std::io::Error),
}
