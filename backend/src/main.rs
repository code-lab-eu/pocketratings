#![allow(clippy::multiple_crate_versions)]

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config =
        pocketratings::config::Config::from_env().context("failed to load configuration")?;

    let pool = pocketratings::db::create_pool(&config.database_path)
        .await
        .context("failed to create database pool")?;

    pocketratings::db::run_migrations(&pool)
        .await
        .context("failed to run database migrations")?;

    tracing::info!("database initialized at {}", config.database_path);

    Ok(())
}
