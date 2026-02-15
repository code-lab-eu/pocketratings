//! CLI commands and parsing.

mod user;

use std::io::Write;

use clap::{CommandFactory, Parser, Subcommand};
use sqlx::SqlitePool;

use crate::cli::user::register;

/// Pocket Ratings — product reviews and ratings.
#[derive(Parser)]
#[command(name = "pocketratings")]
#[command(about = "Product reviews and ratings — backend API and CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    User(UserArgs),
}

/// User subcommand group.
#[derive(clap::Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub command: UserCmd,
}

#[derive(Subcommand)]
pub enum UserCmd {
    Register(RegisterOpts),
}

#[derive(clap::Args)]
pub struct RegisterOpts {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub email: String,
    #[arg(long)]
    pub password: String,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

/// CLI-specific errors for user-facing messages and exit codes.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("email already registered")]
    EmailAlreadyRegistered,

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl From<crate::db::DbError> for CliError {
    fn from(e: crate::db::DbError) -> Self {
        Self::Other(e.into())
    }
}

/// Parse args and dispatch to the appropriate handler.
///
/// When the command is `user register`, `pool` must be `Some`; the caller (e.g. `main`) is responsible for creating the pool and running migrations first.
///
/// # Errors
///
/// Returns [`CliError`] on parse failure, missing pool for `user register`, register handler errors, or I/O when writing help or output.
pub async fn run(
    args: impl Iterator<Item = impl Into<std::ffi::OsString> + Clone>,
    pool: Option<&SqlitePool>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> Result<(), CliError> {
    let cli = Cli::parse_from(args);

    match cli.command {
        Some(Commands::User(user_args)) => match user_args.command {
            UserCmd::Register(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for user register"))
                })?;
                let output_json = opts.output.as_str() == "json";
                register(
                    pool,
                    &opts.name,
                    &opts.email,
                    &opts.password,
                    output_json,
                    stdout,
                    stderr,
                )
                .await
            }
        },
        None => {
            let mut out = Vec::new();
            Cli::command().write_help(&mut out).map_err(|e: std::io::Error| CliError::Other(e.into()))?;
            stdout.write_all(&out).map_err(|e| CliError::Other(e.into()))?;
            Ok(())
        }
    }
}
