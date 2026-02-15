//! User subcommands (e.g. register).

use std::io::Write;

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::auth::password;
use crate::cli::CliError;
use crate::db;
use crate::domain::user::{User, ValidationError};

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

    let hash = password::hash_password(plain_password).map_err(|e| CliError::Validation(e.to_string()))?;
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
        ValidationError::EmailInvalid(s) => CliError::Validation(format!("invalid email address: {s}")),
        ValidationError::PasswordEmpty => CliError::Validation("password hash must not be empty".to_string()),
        ValidationError::CreatedAfterUpdated { .. } | ValidationError::CreatedAfterDeleted { .. } => {
            CliError::Validation(e.to_string())
        }
    })?;

    db::user::insert(pool, &user).await?;

    if output_json {
        let out = serde_json::json!({
            "id": user.id().to_string(),
            "email": user.email(),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(stdout, "User registered: {}", user.email()).map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}
