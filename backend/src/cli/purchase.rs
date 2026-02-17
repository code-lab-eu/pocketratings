//! Purchase subcommands (create, list, show, delete).

use std::io::Write;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::cli::CliError;
use crate::db;
use crate::domain::purchase::{Purchase, ValidationError};

fn map_validation_error(e: &ValidationError) -> CliError {
    match e {
        ValidationError::QuantityInvalid { .. } => {
            CliError::Validation("quantity must be at least 1".to_string())
        }
        ValidationError::PriceInvalid { .. } => {
            CliError::Validation("price must not be negative".to_string())
        }
    }
}

/// Resolve user id from either --user-id or --email.
async fn resolve_user_id(
    pool: &SqlitePool,
    user_id: Option<&str>,
    email: Option<&str>,
) -> Result<Uuid, CliError> {
    match (user_id, email) {
        (Some(id), None) => {
            let uuid = Uuid::parse_str(id)
                .map_err(|_| CliError::Validation(format!("invalid user id: {id}")))?;
            let user = db::user::get_by_id(pool, uuid).await?;
            user.map(|u| u.id())
                .ok_or_else(|| CliError::Validation(format!("user not found: {id}")))
        }
        (None, Some(em)) => {
            let user = db::user::get_by_email(pool, em).await?;
            user.map(|u| u.id())
                .ok_or_else(|| CliError::Validation(format!("user not found for email: {em}")))
        }
        (None, None) => Err(CliError::Validation(
            "either --user-id or --email is required to identify the purchaser".to_string(),
        )),
        (Some(_), Some(_)) => Err(CliError::Validation(
            "provide only one of --user-id or --email".to_string(),
        )),
    }
}

/// Parse ISO 8601 or YYYY-MM-DD date string into UNIX timestamp.
fn parse_date(s: &str) -> Result<i64, CliError> {
    let ts = DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.timestamp())
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                .map(|n| n.and_utc().timestamp())
        })
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
        })
        .map_err(|_| {
            CliError::Validation(format!("invalid date: {s} (use ISO 8601 or YYYY-MM-DD)"))
        })?;
    Ok(ts)
}

/// Parse optional ISO 8601 date string or "now" into UNIX timestamp (for create --at).
fn parse_optional_at(s: Option<&str>) -> Result<i64, CliError> {
    match s {
        None | Some("now") => Ok(Utc::now().timestamp()),
        Some(x) => parse_date(x),
    }
}

/// Create a new purchase.
#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &SqlitePool,
    product_id_str: &str,
    location_id_str: &str,
    price_str: &str,
    user_id: Option<&str>,
    email: Option<&str>,
    quantity: i32,
    at: Option<&str>,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let product_id = Uuid::parse_str(product_id_str)
        .map_err(|_| CliError::Validation(format!("invalid product id: {product_id_str}")))?;
    let Some(_product) = db::product::get_by_id(pool, product_id).await? else {
        return Err(CliError::Validation(format!(
            "product not found: {product_id_str}"
        )));
    };

    let location_id = Uuid::parse_str(location_id_str)
        .map_err(|_| CliError::Validation(format!("invalid location id: {location_id_str}")))?;
    let Some(_location) = db::location::get_by_id(pool, location_id).await? else {
        return Err(CliError::Validation(format!(
            "location not found: {location_id_str}"
        )));
    };

    let user_id_resolved = resolve_user_id(pool, user_id, email).await?;

    let price: Decimal = price_str
        .parse()
        .map_err(|_| CliError::Validation(format!("invalid price: {price_str}")))?;

    let purchased_at = parse_optional_at(at)?;

    let purchase = Purchase::new(
        Uuid::new_v4(),
        user_id_resolved,
        product_id,
        location_id,
        quantity,
        price,
        purchased_at,
        None,
    )
    .map_err(|e| map_validation_error(&e))?;

    db::purchase::insert(pool, &purchase).await?;

    if output_json {
        let out = serde_json::json!({
            "id": purchase.id().to_string(),
            "user_id": purchase.user_id().to_string(),
            "product_id": purchase.product_id().to_string(),
            "location_id": purchase.location_id().to_string(),
            "quantity": purchase.quantity(),
            "price": purchase.price().to_string(),
            "purchased_at": purchase.purchased_at(),
            "deleted": false,
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(
            stdout,
            "Purchase created: {} (qty: {}, price: {})",
            purchase.id(),
            purchase.quantity(),
            purchase.price()
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// List purchases with optional filters.
#[allow(clippy::too_many_arguments)]
pub async fn list(
    pool: &SqlitePool,
    user_id: Option<&str>,
    product_id: Option<&str>,
    location_id: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
    output_json: bool,
    include_deleted: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let user_uuid = user_id
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| CliError::Validation("invalid user_id".to_string()))?;
    let product_uuid = product_id
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| CliError::Validation("invalid product_id".to_string()))?;
    let location_uuid = location_id
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| CliError::Validation("invalid location_id".to_string()))?;
    let from_ts = from.map(parse_date).transpose()?;
    let to_ts = to.map(parse_date).transpose()?;

    let purchases = db::purchase::list(
        pool,
        user_uuid,
        product_uuid,
        location_uuid,
        from_ts,
        to_ts,
        include_deleted,
    )
    .await?;

    if output_json {
        let items: Vec<serde_json::Value> = purchases
            .iter()
            .map(|p| {
                serde_json::json!({
                    "id": p.id().to_string(),
                    "user_id": p.user_id().to_string(),
                    "product_id": p.product_id().to_string(),
                    "location_id": p.location_id().to_string(),
                    "quantity": p.quantity(),
                    "price": p.price().to_string(),
                    "purchased_at": p.purchased_at(),
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
        for p in &purchases {
            writeln!(stdout, "{p}").map_err(|e| CliError::Other(e.into()))?;
        }
    }

    Ok(())
}

/// Show a single purchase by id.
pub async fn show(
    pool: &SqlitePool,
    id_str: &str,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid purchase id: {id_str}")))?;

    let Some(purchase) = db::purchase::get_by_id(pool, id).await? else {
        return Err(CliError::Validation(format!(
            "purchase not found: {id_str}"
        )));
    };

    if output_json {
        let out = serde_json::json!({
            "id": purchase.id().to_string(),
            "user_id": purchase.user_id().to_string(),
            "product_id": purchase.product_id().to_string(),
            "location_id": purchase.location_id().to_string(),
            "quantity": purchase.quantity(),
            "price": purchase.price().to_string(),
            "purchased_at": purchase.purchased_at(),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(stdout, "Purchase: {purchase}").map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// Delete a purchase by id (soft-delete or hard delete with `--force`).
pub async fn delete(
    pool: &SqlitePool,
    id_str: &str,
    force: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid purchase id: {id_str}")))?;
    if force {
        db::purchase::hard_delete(pool, id).await?;
        writeln!(stdout, "Purchase removed: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        db::purchase::soft_delete(pool, id).await?;
        writeln!(stdout, "Purchase deleted: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    }
    Ok(())
}
