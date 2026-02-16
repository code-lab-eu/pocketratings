//! Category subcommands (create, list, show, update, delete).

use std::io::Write;

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::cli::CliError;
use crate::db;
use crate::domain::category::{Category, ValidationError};

fn map_validation_error(e: &ValidationError) -> CliError {
    match e {
        ValidationError::NameEmpty => CliError::Validation("name must not be empty".to_string()),
        ValidationError::CreatedAfterUpdated { .. }
        | ValidationError::CreatedAfterDeleted { .. } => CliError::Validation(e.to_string()),
    }
}

/// Create a new category.
pub async fn create(
    pool: &SqlitePool,
    name: &str,
    parent_id_str: Option<&str>,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let parent_id = match parent_id_str {
        Some(s) => Some(
            Uuid::parse_str(s)
                .map_err(|_| CliError::Validation(format!("invalid parent_id: {s}")))?,
        ),
        None => None,
    };

    let now = Utc::now().timestamp();
    let category = Category::new(Uuid::new_v4(), parent_id, name.to_string(), now, now, None)
        .map_err(|e| map_validation_error(&e))?;

    db::category::insert(pool, &category).await?;

    if output_json {
        let out = serde_json::json!({
            "id": category.id().to_string(),
            "name": category.name(),
            "parent_id": category.parent_id().map(|id| id.to_string()),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(
            stdout,
            "Category created: {} ({})",
            category.name(),
            category.id()
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// List categories (optionally by parent, optionally including soft-deleted).
pub async fn list(
    pool: &SqlitePool,
    parent_id_str: Option<&str>,
    output_json: bool,
    include_deleted: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let parent_id = match parent_id_str {
        Some(s) => Some(
            Uuid::parse_str(s)
                .map_err(|_| CliError::Validation(format!("invalid parent_id: {s}")))?,
        ),
        None => None,
    };

    let mut cats = if include_deleted {
        db::category::get_all_with_deleted(pool).await?
    } else {
        db::category::get_all(pool).await?
    };

    if let Some(pid) = parent_id {
        cats.retain(|c| c.parent_id() == Some(pid));
    }

    // Sort by name for stable output.
    cats.sort_by_key(|c| c.name().to_string());

    if output_json {
        let items: Vec<serde_json::Value> = cats
            .iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.id().to_string(),
                    "name": c.name(),
                    "parent_id": c.parent_id().map(|id| id.to_string()),
                    "deleted": !c.is_active(),
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
        for c in &cats {
            let parent = c
                .parent_id()
                .map_or_else(|| "root".to_string(), |id| id.to_string());
            writeln!(stdout, "{}  {}  parent: {}", c.id(), c.name(), parent)
                .map_err(|e| CliError::Other(e.into()))?;
        }
    }

    Ok(())
}

/// Show a single category by id.
pub async fn show(
    pool: &SqlitePool,
    id_str: &str,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid category id: {id_str}")))?;

    let Some(cat) = db::category::get_by_id(pool, id).await? else {
        return Err(CliError::Validation(format!(
            "category not found: {id_str}"
        )));
    };

    if output_json {
        let out = serde_json::json!({
            "id": cat.id().to_string(),
            "name": cat.name(),
            "parent_id": cat.parent_id().map(|pid| pid.to_string()),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        let parent = cat
            .parent_id()
            .map_or_else(|| "root".to_string(), |id| id.to_string());
        writeln!(
            stdout,
            "Category {}: {} (parent: {})",
            cat.id(),
            cat.name(),
            parent
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// Update category name and/or parent.
pub async fn update(
    pool: &SqlitePool,
    id_str: &str,
    name: Option<&str>,
    parent_id_str: Option<&str>,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid category id: {id_str}")))?;

    let Some(existing) = db::category::get_by_id(pool, id).await? else {
        return Err(CliError::Validation(format!(
            "category not found: {id_str}"
        )));
    };

    let new_name = name.unwrap_or_else(|| existing.name()).to_string();
    let new_parent_id = match parent_id_str {
        Some(s) => Some(
            Uuid::parse_str(s)
                .map_err(|_| CliError::Validation(format!("invalid parent_id: {s}")))?,
        ),
        None => existing.parent_id(),
    };
    let now = Utc::now().timestamp();

    let updated = Category::new(
        existing.id(),
        new_parent_id,
        new_name,
        existing.created_at(),
        now,
        existing.deleted_at(),
    )
    .map_err(|e| map_validation_error(&e))?;

    db::category::update(pool, &updated).await?;

    if output_json {
        let out = serde_json::json!({
            "id": updated.id().to_string(),
            "name": updated.name(),
            "parent_id": updated.parent_id().map(|pid| pid.to_string()),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        let parent = updated
            .parent_id()
            .map_or_else(|| "root".to_string(), |id| id.to_string());
        writeln!(
            stdout,
            "Category updated: {} (parent: {})",
            updated.name(),
            parent
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// Delete a category by id (soft-delete or hard delete with `--force`).
pub async fn delete(
    pool: &SqlitePool,
    id_str: &str,
    force: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid category id: {id_str}")))?;
    if force {
        db::category::hard_delete(pool, id).await?;
        writeln!(stdout, "Category removed: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        db::category::soft_delete(pool, id).await?;
        writeln!(stdout, "Category deleted: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    }
    Ok(())
}
