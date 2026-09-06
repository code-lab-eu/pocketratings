//! User subcommands (e.g. register, list, delete).

use std::io::Write;

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::auth::password;
use crate::auth::reset_token;
use crate::cli::CliError;
use crate::config::Config;
use crate::db;
use crate::domain::user::{User, ValidationError};

/// Delete a user by id (soft-delete or hard delete with `force`). Writes a success message to stdout.
pub async fn delete(
    pool: &SqlitePool,
    id_str: &str,
    force: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid user id: {id_str}")))?;
    if force {
        db::user::hard_delete(pool, id).await?;
        writeln!(stdout, "User removed: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        db::user::soft_delete(pool, id).await?;
        writeln!(stdout, "User deleted: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    }
    Ok(())
}

/// Look up an active user by email, or fail with the standard "user not found" message.
async fn require_user_by_email(pool: &SqlitePool, email: &str) -> Result<User, CliError> {
    db::user::get_by_email(pool, email)
        .await?
        .ok_or_else(|| CliError::Validation(format!("user not found: {email}")))
}

/// Change a user's password: look up by email, hash the new password, update the stored hash.
/// Writes a success message to stdout.
pub async fn set_password(
    pool: &SqlitePool,
    email: &str,
    plain_password: &str,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let user = require_user_by_email(pool, email).await?;

    let hash =
        password::hash_password(plain_password).map_err(|e| CliError::Validation(e.to_string()))?;
    db::user::update_password(pool, user.id(), &hash).await?;

    writeln!(stdout, "Password updated for: {email}").map_err(|e| CliError::Other(e.into()))?;
    Ok(())
}

/// Create a one-time password reset link for a user and print it.
///
/// Only the token hash is stored, so the link cannot be recovered from the database; print it
/// once and hand it to the user over a secure channel.
pub async fn reset_link(
    pool: &SqlitePool,
    config_override: Option<&Config>,
    email: &str,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let config = match config_override {
        Some(c) => c.clone(),
        None => Config::from_env().map_err(|e| CliError::Other(e.into()))?,
    };

    let user = require_user_by_email(pool, email).await?;

    let token = reset_token::generate();
    let now = Utc::now().timestamp();
    db::password_reset::create(pool, user.id(), &reset_token::hash(&token), now).await?;

    let url = format!("{}/reset-password?token={token}", config.app_base_url);
    let expires_at = now + reset_token::TOKEN_TTL_SECONDS;

    if output_json {
        let out = serde_json::json!({ "url": url, "expires_at": expires_at });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        let until = DateTime::from_timestamp(expires_at, 0)
            .map_or_else(|| expires_at.to_string(), |t| t.to_rfc3339());
        writeln!(stdout, "{url}").map_err(|e| CliError::Other(e.into()))?;
        writeln!(stdout, "Valid for 12 hours (until {until}).")
            .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// List users: fetch from DB, write to stdout (human or JSON).
pub async fn list(
    pool: &SqlitePool,
    output_json: bool,
    include_deleted: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let users = db::user::list_all(pool, include_deleted).await?;

    if output_json {
        let items: Vec<serde_json::Value> = users
            .iter()
            .map(|u| {
                serde_json::json!({
                    "id": u.id().to_string(),
                    "email": u.email(),
                    "name": u.name(),
                })
            })
            .collect();
        writeln!(
            stdout,
            "{}",
            serde_json::to_string(&items).map_err(|e| CliError::Other(e.into()))?
        )
        .map_err(|e| CliError::Other(e.into()))?;
    } else {
        for u in &users {
            writeln!(stdout, "{}  {}  {}", u.id(), u.email(), u.name())
                .map_err(|e| CliError::Other(e.into()))?;
        }
    }

    Ok(())
}

/// Register a new user: check email uniqueness, hash password, insert, write result to stdout.
pub async fn register(
    pool: &SqlitePool,
    name: &str,
    email: &str,
    plain_password: &str,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    if db::user::get_by_email(pool, email).await?.is_some() {
        return Err(CliError::EmailAlreadyRegistered);
    }

    let hash =
        password::hash_password(plain_password).map_err(|e| CliError::Validation(e.to_string()))?;
    let now = Utc::now().timestamp();
    let user = User::new(
        Uuid::new_v4(),
        name.to_string(),
        email.to_string(),
        hash,
        now,
        now,
        None,
    )
    .map_err(|e| match e {
        ValidationError::NameEmpty => CliError::Validation("name must not be empty".to_string()),
        ValidationError::EmailInvalid(s) => {
            CliError::Validation(format!("invalid email address: {s}"))
        }
        ValidationError::PasswordEmpty => {
            CliError::Validation("password hash must not be empty".to_string())
        }
        ValidationError::CreatedAfterUpdated { .. }
        | ValidationError::CreatedAfterDeleted { .. } => CliError::Validation(e.to_string()),
    })?;

    db::user::insert(pool, &user).await?;

    if output_json {
        let out = serde_json::json!({
            "id": user.id().to_string(),
            "email": user.email(),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(stdout, "User registered: {}", user.email())
            .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}
