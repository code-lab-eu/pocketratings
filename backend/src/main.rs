#![allow(clippy::multiple_crate_versions)]

use std::io::Write;

use anyhow::Context;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let needs_db = args
        .get(1)
        .and_then(|a| a.to_str())
        .is_some_and(|s| s == "user")
        && args
            .get(2)
            .and_then(|a| a.to_str())
            .is_some_and(|s| s == "register" || s == "list" || s == "delete");

    let pool = if needs_db {
        let config =
            pocketratings::config::Config::from_env().context("failed to load configuration");
        match config {
            Ok(c) => {
                match pocketratings::db::create_pool(&c.database_path).await {
                    Ok(p) => match pocketratings::db::run_migrations(&p).await {
                        Ok(()) => Some(p),
                        Err(e) => {
                            let _ = writeln!(
                                std::io::stderr(),
                                "{}",
                                anyhow::anyhow!(e).context("failed to run database migrations")
                            );
                            std::process::exit(1);
                        }
                    },
                    Err(e) => {
                        let _ = writeln!(
                            std::io::stderr(),
                            "{}",
                            anyhow::anyhow!(e).context("failed to create database pool")
                        );
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                let _ = writeln!(std::io::stderr(), "{e}");
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    let exit_code = match pocketratings::cli::run(
        args.into_iter(),
        pool.as_ref(),
        &mut stdout,
        &mut stderr,
    )
    .await
    {
        Ok(()) => 0,
        Err(e) => {
            let _ = writeln!(stderr, "{e}");
            1
        }
    };
    std::process::exit(exit_code);
}
