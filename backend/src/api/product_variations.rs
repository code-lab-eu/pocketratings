//! Product variations REST API: list, create, update, delete.
//!
//! Handlers and types for GET/POST /api/v1/products/:id/variations and
//! PATCH/DELETE /api/v1/variations/:id. Used by product detail (variations
//! included in GET product) and by the variation list endpoint.

#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::{error::ApiError, state::AppState};
use crate::db;
use crate::domain::product_variation::ProductVariation;

/// One variation in list response (GET /api/v1/products/:id/variations) and in
/// product detail variations array.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VariationListItem {
    pub id: Uuid,
    pub label: String,
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
    /// Number of non-deleted purchases referencing this variation (for edit-product UI).
    pub purchase_count: u64,
}

/// Request body for creating a variation (POST /api/v1/products/:id/variations).
#[derive(Debug, Deserialize)]
pub struct CreateVariationRequest {
    pub label: Option<String>,
    pub unit: String,
    pub quantity: Option<u32>,
}

/// Request body for updating a variation (PATCH /api/v1/variations/:id).
/// `quantity`: omit = keep existing, null = clear, number = set.
#[derive(Debug, Deserialize)]
#[allow(clippy::option_option)]
pub struct UpdateVariationRequest {
    pub label: Option<String>,
    pub unit: Option<String>,
    pub quantity: Option<Option<u32>>,
}

#[allow(clippy::needless_pass_by_value)]
fn map_db_error(e: db::DbError) -> ApiError {
    match &e {
        db::DbError::InvalidData(msg) => {
            if msg.contains("cannot delete") {
                ApiError::Conflict(msg.clone())
            } else {
                ApiError::BadRequest(msg.clone())
            }
        }
        db::DbError::Sqlx(sqlx_err) => {
            if let sqlx::Error::Database(db) = sqlx_err
                && (db.is_unique_violation() || db.is_foreign_key_violation())
            {
                return ApiError::BadRequest(e.to_string());
            }
            ApiError::Internal
        }
        db::DbError::Migrate(_) => ApiError::Internal,
    }
}

/// Build variation list with purchase counts for a product. Used by the list
/// variations handler and by product detail (GET /api/v1/products/:id).
pub async fn list_variations_for_product(
    pool: &sqlx::SqlitePool,
    product_id: Uuid,
) -> Result<Vec<VariationListItem>, ApiError> {
    let variations = db::product_variation::list_by_product_id(pool, product_id, false)
        .await
        .map_err(map_db_error)?;
    let ids: Vec<Uuid> = variations.iter().map(ProductVariation::id).collect();
    let counts = db::purchase::count_by_variation_ids(pool, &ids)
        .await
        .map_err(map_db_error)?;
    let list: Vec<VariationListItem> = variations
        .iter()
        .map(|v| {
            let purchase_count = counts.get(&v.id()).copied().unwrap_or(0);
            VariationListItem {
                id: v.id(),
                label: v.label().to_string(),
                unit: v.unit().to_string(),
                quantity: v.quantity(),
                purchase_count: u64::try_from(purchase_count).unwrap_or(0),
            }
        })
        .collect();
    Ok(list)
}

/// GET /api/v1/products/:id/variations — list active variations for a product.
pub async fn list_product_variations(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<VariationListItem>>, ApiError> {
    let _ = db::product::get_by_id(&state.pool, id, false)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| ApiError::NotFound("Product not found.".to_string()))?;
    let list = list_variations_for_product(&state.pool, id).await?;
    Ok(Json(list))
}

/// POST /api/v1/products/:id/variations — create a variation for a product.
pub async fn create_variation(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Json(body): Json<CreateVariationRequest>,
) -> Result<(StatusCode, Json<VariationListItem>), ApiError> {
    let _ = db::product::get_by_id(&state.pool, product_id, false)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| ApiError::NotFound("Product not found.".to_string()))?;
    let label = body.label.as_deref().unwrap_or("").trim();
    let variation = ProductVariation::new(
        Uuid::new_v4(),
        product_id,
        label,
        body.unit.trim(),
        body.quantity,
        chrono::Utc::now().timestamp(),
        chrono::Utc::now().timestamp(),
        None,
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    db::product_variation::insert(&state.pool, &variation)
        .await
        .map_err(map_db_error)?;
    let item = VariationListItem {
        id: variation.id(),
        label: variation.label().to_string(),
        unit: variation.unit().to_string(),
        quantity: variation.quantity(),
        purchase_count: 0,
    };
    Ok((StatusCode::CREATED, Json(item)))
}

/// PATCH /api/v1/variations/:id — update a variation.
pub async fn update_variation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateVariationRequest>,
) -> Result<Json<VariationListItem>, ApiError> {
    let existing = db::product_variation::get_by_id(&state.pool, id, false)
        .await
        .map_err(map_db_error)?;
    let existing =
        existing.ok_or_else(|| ApiError::NotFound("Variation not found.".to_string()))?;
    let label = body
        .label
        .as_deref()
        .map_or_else(|| existing.label().to_string(), |s| s.trim().to_string());
    let unit_str = body
        .unit
        .as_deref()
        .map_or_else(|| existing.unit().to_string(), |s| s.trim().to_string());
    let quantity = body.quantity.unwrap_or_else(|| existing.quantity());
    let updated = ProductVariation::new(
        existing.id(),
        existing.product_id(),
        &label,
        &unit_str,
        quantity,
        existing.created_at(),
        chrono::Utc::now().timestamp(),
        existing.deleted_at(),
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    db::product_variation::update(&state.pool, &updated)
        .await
        .map_err(map_db_error)?;
    let count = db::purchase::count_by_variation_ids(&state.pool, &[id])
        .await
        .map_err(map_db_error)?;
    let purchase_count = count.get(&id).copied().unwrap_or(0);
    let item = VariationListItem {
        id: updated.id(),
        label: updated.label().to_string(),
        unit: updated.unit().to_string(),
        quantity: updated.quantity(),
        purchase_count: u64::try_from(purchase_count).unwrap_or(0),
    };
    Ok(Json(item))
}

/// DELETE /api/v1/variations/:id — soft-delete a variation.
pub async fn delete_variation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let existing = db::product_variation::get_by_id(&state.pool, id, false)
        .await
        .map_err(map_db_error)?;
    let existing =
        existing.ok_or_else(|| ApiError::NotFound("Variation not found.".to_string()))?;
    db::product_variation::ensure_no_purchases(&state.pool, id)
        .await
        .map_err(map_db_error)?;
    let count =
        db::product_variation::count_by_product_id(&state.pool, existing.product_id(), false)
            .await
            .map_err(map_db_error)?;
    if count <= 1 {
        return Err(ApiError::Conflict(
            "Cannot delete the last variation.".to_string(),
        ));
    }
    db::product_variation::soft_delete(&state.pool, id)
        .await
        .map_err(map_db_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Router for product variation endpoints (merge into product router).
pub fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/products/{id}/variations",
            get(list_product_variations).post(create_variation),
        )
        .route(
            "/api/v1/variations/{id}",
            patch(update_variation).delete(delete_variation),
        )
}
