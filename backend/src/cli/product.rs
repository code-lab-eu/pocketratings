//! Product subcommands (create, list, show, update, delete).

use std::collections::HashMap;
use std::io::Write;

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::cli::CliError;
use crate::db;
use crate::domain::category::Category;
use crate::domain::product::{Product, ValidationError};

/// Format category for CLI output: uses [`Category`]'s Display when available, else `uuid (?)`.
fn format_category_display(cat: Option<&Category>, id: Uuid) -> String {
    cat.map_or_else(|| format!("{id} (?)"), std::string::ToString::to_string)
}

fn map_validation_error(e: &ValidationError) -> CliError {
    match e {
        ValidationError::BrandEmpty => CliError::Validation("brand must not be empty".to_string()),
        ValidationError::NameEmpty => CliError::Validation("name must not be empty".to_string()),
        ValidationError::CreatedAfterUpdated { .. }
        | ValidationError::CreatedAfterDeleted { .. } => CliError::Validation(e.to_string()),
    }
}

/// Create a new product.
pub async fn create(
    pool: &SqlitePool,
    name: &str,
    brand: &str,
    category_id_str: &str,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let category_id = Uuid::parse_str(category_id_str)
        .map_err(|_| CliError::Validation(format!("invalid category_id: {category_id_str}")))?;

    let Some(_cat) = db::category::get_by_id(pool, category_id).await? else {
        return Err(CliError::Validation(format!(
            "category not found: {category_id_str}"
        )));
    };

    let now = Utc::now().timestamp();
    let product = Product::new(
        Uuid::new_v4(),
        category_id,
        brand.to_string(),
        name.to_string(),
        now,
        now,
        None,
    )
    .map_err(|e| map_validation_error(&e))?;

    db::product::insert(pool, &product).await?;

    if output_json {
        let out = serde_json::json!({
            "id": product.id().to_string(),
            "brand": product.brand(),
            "name": product.name(),
            "category_id": product.category_id().to_string(),
            "deleted": false,
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(
            stdout,
            "Product created: {} ({})",
            product.name(),
            product.id()
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// List products (optionally by category, optionally including soft-deleted).
pub async fn list(
    pool: &SqlitePool,
    category_id_str: Option<&str>,
    output_json: bool,
    include_deleted: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let mut products = if let Some(s) = category_id_str {
        let category_id = Uuid::parse_str(s)
            .map_err(|_| CliError::Validation(format!("invalid category_id: {s}")))?;
        if include_deleted {
            db::product::get_all_by_category_id_with_deleted(pool, category_id).await?
        } else {
            db::product::get_all_by_category_id(pool, category_id).await?
        }
    } else if include_deleted {
        db::product::get_all_with_deleted(pool).await?
    } else {
        db::product::get_all(pool).await?
    };

    // Sort by (brand, name) for stable output.
    products.sort_by(|a, b| (a.brand(), a.name()).cmp(&(b.brand(), b.name())));

    let category_map: HashMap<Uuid, Category> = db::category::get_all(pool, include_deleted)
        .await?
        .into_iter()
        .map(|c| (c.id(), c))
        .collect();

    if output_json {
        let items: Vec<serde_json::Value> = products
            .iter()
            .map(|p| {
                let category_name = category_map
                    .get(&p.category_id())
                    .map(|c| c.name().to_string());
                serde_json::json!({
                    "id": p.id().to_string(),
                    "brand": p.brand(),
                    "name": p.name(),
                    "category_id": p.category_id().to_string(),
                    "category_name": category_name,
                    "deleted": !p.is_active(),
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
        for p in &products {
            let cat = category_map.get(&p.category_id());
            writeln!(
                stdout,
                "{}  {}  {}  category: {}",
                p.id(),
                p.brand(),
                p.name(),
                format_category_display(cat, p.category_id())
            )
            .map_err(|e| CliError::Other(e.into()))?;
        }
    }

    Ok(())
}

/// Show a single product by id.
pub async fn show(
    pool: &SqlitePool,
    id_str: &str,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid product id: {id_str}")))?;

    let Some(product) = db::product::get_by_id(pool, id).await? else {
        return Err(CliError::Validation(format!("product not found: {id_str}")));
    };

    let category = db::category::get_by_id(pool, product.category_id()).await?;

    if output_json {
        let category_name = category.as_ref().map(|c| c.name().to_string());
        let out = serde_json::json!({
            "id": product.id().to_string(),
            "brand": product.brand(),
            "name": product.name(),
            "category_id": product.category_id().to_string(),
            "category_name": category_name,
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(
            stdout,
            "Product {}: {} ({})  category: {}",
            product.id(),
            product.name(),
            product.brand(),
            format_category_display(category.as_ref(), product.category_id())
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// Update product name, brand, and/or category.
#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &SqlitePool,
    id_str: &str,
    name: Option<&str>,
    brand: Option<&str>,
    category_id_str: Option<&str>,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid product id: {id_str}")))?;

    let Some(existing) = db::product::get_by_id(pool, id).await? else {
        return Err(CliError::Validation(format!("product not found: {id_str}")));
    };

    let new_name = name.unwrap_or_else(|| existing.name()).to_string();
    let new_brand = brand.unwrap_or_else(|| existing.brand()).to_string();
    let new_category_id = match category_id_str {
        Some(s) => Uuid::parse_str(s)
            .map_err(|_| CliError::Validation(format!("invalid category_id: {s}")))?,
        None => existing.category_id(),
    };
    let now = Utc::now().timestamp();

    let updated = Product::new(
        existing.id(),
        new_category_id,
        new_brand,
        new_name,
        existing.created_at(),
        now,
        existing.deleted_at(),
    )
    .map_err(|e| map_validation_error(&e))?;

    db::product::update(pool, &updated).await?;

    if output_json {
        let out = serde_json::json!({
            "id": updated.id().to_string(),
            "brand": updated.brand(),
            "name": updated.name(),
            "category_id": updated.category_id().to_string(),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(
            stdout,
            "Product updated: {} ({})",
            updated.name(),
            updated.brand()
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// Delete a product by id (soft-delete or hard delete with `--force`).
pub async fn delete(
    pool: &SqlitePool,
    id_str: &str,
    force: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid product id: {id_str}")))?;
    if force {
        db::product::hard_delete(pool, id).await?;
        writeln!(stdout, "Product removed: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        db::product::soft_delete(pool, id).await?;
        writeln!(stdout, "Product deleted: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    }
    Ok(())
}
