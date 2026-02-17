//! Review subcommands (create, list, show, update, delete).

use std::io::Write;

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::cli::CliError;
use crate::db;
use crate::domain::review::{Review, ValidationError};

fn map_validation_error(e: &ValidationError) -> CliError {
    match e {
        ValidationError::RatingOutOfRange { .. } => {
            CliError::Validation("rating must be between 1 and 5".to_string())
        }
        ValidationError::CreatedAfterUpdated { .. }
        | ValidationError::CreatedAfterDeleted { .. } => CliError::Validation(e.to_string()),
    }
}

/// Resolve user id from either --user-id or --email. Returns error if neither/both or not found.
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
            "either --user-id or --email is required to identify the reviewer".to_string(),
        )),
        (Some(_), Some(_)) => Err(CliError::Validation(
            "provide only one of --user-id or --email".to_string(),
        )),
    }
}

/// Create a new review.
#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &SqlitePool,
    product_id_str: &str,
    rating_str: &str,
    user_id: Option<&str>,
    email: Option<&str>,
    text: Option<&str>,
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

    let user_id_resolved = resolve_user_id(pool, user_id, email).await?;

    let rating: Decimal = rating_str
        .parse()
        .map_err(|_| CliError::Validation(format!("invalid rating: {rating_str} (must be 1-5)")))?;

    let now = Utc::now().timestamp();
    let review = Review::new(
        Uuid::new_v4(),
        product_id,
        user_id_resolved,
        rating,
        text.map(str::to_owned),
        now,
        now,
        None,
    )
    .map_err(|e| map_validation_error(&e))?;

    db::review::insert(pool, &review).await?;

    if output_json {
        let out = serde_json::json!({
            "id": review.id().to_string(),
            "product_id": review.product_id().to_string(),
            "user_id": review.user_id().to_string(),
            "rating": review.rating().to_string(),
            "text": review.text(),
            "deleted": false,
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(
            stdout,
            "Review created: {} (rating: {})",
            review.id(),
            review.rating()
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// List reviews with optional filters.
pub async fn list(
    pool: &SqlitePool,
    product_id: Option<&str>,
    user_id: Option<&str>,
    output_json: bool,
    include_deleted: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let product_uuid = product_id.map(Uuid::parse_str).transpose().map_err(|_| {
        CliError::Validation(format!(
            "invalid product_id: {}",
            product_id.unwrap_or_default()
        ))
    })?;
    let user_uuid = user_id.map(Uuid::parse_str).transpose().map_err(|_| {
        CliError::Validation(format!("invalid user_id: {}", user_id.unwrap_or_default()))
    })?;

    let reviews = db::review::list(pool, product_uuid, user_uuid, include_deleted).await?;

    if output_json {
        let items: Vec<serde_json::Value> = reviews
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id().to_string(),
                    "product_id": r.product_id().to_string(),
                    "user_id": r.user_id().to_string(),
                    "rating": r.rating().to_string(),
                    "text": r.text(),
                    "deleted": !r.is_active(),
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
        for r in &reviews {
            writeln!(stdout, "{r}").map_err(|e| CliError::Other(e.into()))?;
        }
    }

    Ok(())
}

/// Show a single review by id.
pub async fn show(
    pool: &SqlitePool,
    id_str: &str,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid review id: {id_str}")))?;

    let Some(review) = db::review::get_by_id(pool, id).await? else {
        return Err(CliError::Validation(format!("review not found: {id_str}")));
    };

    if output_json {
        let out = serde_json::json!({
            "id": review.id().to_string(),
            "product_id": review.product_id().to_string(),
            "user_id": review.user_id().to_string(),
            "rating": review.rating().to_string(),
            "text": review.text(),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(stdout, "Review: {review}").map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// Update a review (rating and/or text).
pub async fn update(
    pool: &SqlitePool,
    id_str: &str,
    rating: Option<&str>,
    text: Option<&str>,
    output_json: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid review id: {id_str}")))?;

    let Some(existing) = db::review::get_by_id(pool, id).await? else {
        return Err(CliError::Validation(format!("review not found: {id_str}")));
    };

    let new_rating = rating
        .map(str::parse::<Decimal>)
        .transpose()
        .map_err(|_| CliError::Validation("invalid rating (must be 1-5)".to_string()))?
        .unwrap_or_else(|| existing.rating());
    let new_text = text
        .map(str::to_owned)
        .or_else(|| existing.text().map(str::to_owned));
    let now = Utc::now().timestamp();

    let updated = Review::new(
        existing.id(),
        existing.product_id(),
        existing.user_id(),
        new_rating,
        new_text,
        existing.created_at(),
        now,
        existing.deleted_at(),
    )
    .map_err(|e| map_validation_error(&e))?;

    db::review::update(pool, &updated).await?;

    if output_json {
        let out = serde_json::json!({
            "id": updated.id().to_string(),
            "product_id": updated.product_id().to_string(),
            "user_id": updated.user_id().to_string(),
            "rating": updated.rating().to_string(),
            "text": updated.text(),
        });
        writeln!(stdout, "{out}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        writeln!(
            stdout,
            "Review updated: {} (rating: {})",
            updated.id(),
            updated.rating()
        )
        .map_err(|e| CliError::Other(e.into()))?;
    }

    Ok(())
}

/// Delete a review by id (soft-delete or hard delete with `--force`).
pub async fn delete(
    pool: &SqlitePool,
    id_str: &str,
    force: bool,
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> Result<(), CliError> {
    let id = Uuid::parse_str(id_str)
        .map_err(|_| CliError::Validation(format!("invalid review id: {id_str}")))?;
    if force {
        db::review::hard_delete(pool, id).await?;
        writeln!(stdout, "Review removed: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    } else {
        db::review::soft_delete(pool, id).await?;
        writeln!(stdout, "Review deleted: {id_str}").map_err(|e| CliError::Other(e.into()))?;
    }
    Ok(())
}
