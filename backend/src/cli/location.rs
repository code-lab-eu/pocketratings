//! Location subcommands (create, list, show, update, delete).

use std::io::Write;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::cli::CliError;
use crate::db;
use crate::domain::location::{Location, ValidationError};

fn map_validation_error(e: &ValidationError) -> CliError {
    match e {
        ValidationError::NameEmpty => CliError::Validation("name must not be empty".to_string()),
    }
}

/// Create a new location.
pub async fn create(
    pool: &SqlitePool,
    name: &str,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let location = Location::new(Uuid::new_v4(), name.to_string(), None)
        .map_err(|e| map_validation_error(&e))?;

    db::location::insert(pool, &location).await?;

    if output_json {
        let out = serde_json::json!({
            "id": location.id().to_string(),
            "name": location.name(),
            "deleted": false,
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(
            stdout,
            "Location created: {} ({})",
            location.name(),
            location.id()
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// List locations (optionally including soft-deleted).
pub async fn list(
    pool: &SqlitePool,
    output_json: bool,
    include_deleted: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let mut locations = if include_deleted {
        db::location::get_all_with_deleted(pool).await?
    } else {
        db::location::get_all(pool).await?
    };

    locations.sort_by_key(|l| l.name().to_string());

    if output_json {
        let items: Vec<serde_json::Value> = locations
            .iter()
            .map(|l| {
                serde_json::json!({
                    "id": l.id().to_string(),
                    "name": l.name(),
                    "deleted": !l.is_active(),
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
        for l in &locations {
            writeln!(stdout, "{l}").map_err(|e| CliError::Other(e.into()))?;
        }
    }

    Ok(())
}

/// Show a single location by id.
pub async fn show(
    pool: &SqlitePool,
    id_str: &str,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid location id: {id_str}")))?;

    let Some(location) = db::location::get_by_id(pool, id).await? else {
        return Err(CliError::Validation(format!(
            "location not found: {id_str}"
        )));
    };

    if output_json {
        let out = serde_json::json!({
            "id": location.id().to_string(),
            "name": location.name(),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(stdout, "Location: {location}").map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// Update location name.
pub async fn update(
    pool: &SqlitePool,
    id_str: &str,
    name: Option<&str>,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid location id: {id_str}")))?;

    let Some(existing) = db::location::get_by_id(pool, id).await? else {
        return Err(CliError::Validation(format!(
            "location not found: {id_str}"
        )));
    };

    let new_name = name.unwrap_or_else(|| existing.name()).to_string();
    let updated = Location::new(existing.id(), new_name, existing.deleted_at())
        .map_err(|e| map_validation_error(&e))?;

    db::location::update(pool, &updated).await?;

    if output_json {
        let out = serde_json::json!({
            "id": updated.id().to_string(),
            "name": updated.name(),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(
            stdout,
            "Location updated: {} ({})",
            updated.name(),
            updated.id()
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// Delete a location by id (soft-delete or hard delete with `--force`).
pub async fn delete(
    pool: &SqlitePool,
    id_str: &str,
    force: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid location id: {id_str}")))?;
    if force {
        db::location::hard_delete(pool, id).await?;
        writeln!(stdout, "Location removed: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        db::location::soft_delete(pool, id).await?;
        writeln!(stdout, "Location deleted: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    }
    Ok(())
}
