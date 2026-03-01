#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

use std::io::{IsTerminal, Write};

use anyhow::Context;

/// When stderr is not a TTY (e.g. piped in tests), flush after each write so log lines
/// are visible to the reader without waiting for a full buffer.
struct StderrWriter;

impl Write for StderrWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut w = std::io::stderr();
        let n = w.write(buf)?;
        if !w.is_terminal() {
            w.flush()?;
        }
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stderr().flush()
    }
}

#[tokio::main]
async fn main() {
    pocketratings::db::category::set_running_as_production();
    pocketratings::db::review::set_running_as_production();
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_writer(|| StderrWriter)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let first = args.get(1).and_then(|a| a.to_str());
    let second = args.get(2).and_then(|a| a.to_str());
    let needs_db = matches!(
        (first, second),
        (Some("user"), Some("register" | "list" | "delete"))
            | (
                Some("category" | "location" | "product" | "purchase" | "review"),
                Some("create" | "list" | "show" | "update" | "delete")
            )
            | (Some("server"), Some("start"))
            | (Some("database"), Some("backup"))
    );

    let pool = if needs_db {
        let config =
            pocketratings::config::Config::from_env().context("failed to load configuration");
        match config {
            Ok(c) => match pocketratings::db::create_pool(&c.database_path).await {
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
            },
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
        None,
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
