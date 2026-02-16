//! Product domain type with field validation.

use uuid::Uuid;

/// Validation errors for [`Product`] fields.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// The brand field is empty.
    #[error("brand must not be empty")]
    BrandEmpty,

    /// The name field is empty.
    #[error("name must not be empty")]
    NameEmpty,

    /// `created_at` is after `updated_at`.
    #[error("created_at ({created_at}) must not be after updated_at ({updated_at})")]
    CreatedAfterUpdated {
        /// The `created_at` value.
        created_at: i64,
        /// The `updated_at` value.
        updated_at: i64,
    },

    /// `created_at` is after `deleted_at`.
    #[error("created_at ({created_at}) must not be after deleted_at ({deleted_at})")]
    CreatedAfterDeleted {
        /// The `created_at` value.
        created_at: i64,
        /// The `deleted_at` value.
        deleted_at: i64,
    },
}

/// A validated product (belongs to a category).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Product {
    id: Uuid,
    category_id: Uuid,
    brand: String,
    name: String,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
}

impl Product {
    /// Create a new `Product` after validating all fields.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if any field is invalid.
    pub fn new(
        id: Uuid,
        category_id: Uuid,
        brand: String,
        name: String,
        created_at: i64,
        updated_at: i64,
        deleted_at: Option<i64>,
    ) -> Result<Self, ValidationError> {
        if brand.trim().is_empty() {
            return Err(ValidationError::BrandEmpty);
        }
        if name.trim().is_empty() {
            return Err(ValidationError::NameEmpty);
        }

        if created_at > updated_at {
            return Err(ValidationError::CreatedAfterUpdated {
                created_at,
                updated_at,
            });
        }

        if let Some(deleted) = deleted_at
            && created_at > deleted
        {
            return Err(ValidationError::CreatedAfterDeleted {
                created_at,
                deleted_at: deleted,
            });
        }

        Ok(Self {
            id,
            category_id,
            brand,
            name,
            created_at,
            updated_at,
            deleted_at,
        })
    }

    /// Whether the product is active (not soft-deleted).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// The product's unique identifier.
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// The category this product belongs to.
    #[must_use]
    pub const fn category_id(&self) -> Uuid {
        self.category_id
    }

    /// The product brand.
    #[must_use]
    pub fn brand(&self) -> &str {
        &self.brand
    }

    /// The product name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// UNIX timestamp when the product was created.
    #[must_use]
    pub const fn created_at(&self) -> i64 {
        self.created_at
    }

    /// UNIX timestamp when the product was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> i64 {
        self.updated_at
    }

    /// UNIX timestamp when the product was soft-deleted, if any.
    #[must_use]
    pub const fn deleted_at(&self) -> Option<i64> {
        self.deleted_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_product(
        brand: &str,
        name: &str,
        category_id: Uuid,
        deleted_at: Option<i64>,
    ) -> Result<Product, ValidationError> {
        Product::new(
            Uuid::new_v4(),
            category_id,
            brand.to_owned(),
            name.to_owned(),
            1_000,
            1_000,
            deleted_at,
        )
    }

    #[test]
    fn valid_product() {
        let cat_id = Uuid::new_v4();
        let prod = make_product("Acme", "Widget", cat_id, None);
        assert!(prod.is_ok());
        let prod = prod.unwrap();
        assert!(prod.is_active());
        assert_eq!(prod.brand(), "Acme");
        assert_eq!(prod.name(), "Widget");
        assert_eq!(prod.category_id(), cat_id);
    }

    #[test]
    fn valid_deleted_product() {
        let cat_id = Uuid::new_v4();
        let prod = make_product("Acme", "Widget", cat_id, Some(1_000));
        assert!(prod.is_ok());
        assert!(!prod.unwrap().is_active());
    }

    #[test]
    fn empty_brand_is_rejected() {
        let err = make_product("", "Widget", Uuid::new_v4(), None).unwrap_err();
        assert_eq!(err, ValidationError::BrandEmpty);
    }

    #[test]
    fn whitespace_only_brand_is_rejected() {
        let err = make_product("   ", "Widget", Uuid::new_v4(), None).unwrap_err();
        assert_eq!(err, ValidationError::BrandEmpty);
    }

    #[test]
    fn empty_name_is_rejected() {
        let err = make_product("Acme", "", Uuid::new_v4(), None).unwrap_err();
        assert_eq!(err, ValidationError::NameEmpty);
    }

    #[test]
    fn whitespace_only_name_is_rejected() {
        let err = make_product("Acme", "   ", Uuid::new_v4(), None).unwrap_err();
        assert_eq!(err, ValidationError::NameEmpty);
    }

    #[test]
    fn created_after_updated_is_rejected() {
        let err = Product::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "A".to_owned(),
            "B".to_owned(),
            200,
            100,
            None,
        )
        .unwrap_err();
        assert_eq!(
            err,
            ValidationError::CreatedAfterUpdated {
                created_at: 200,
                updated_at: 100,
            }
        );
    }

    #[test]
    fn created_after_deleted_is_rejected() {
        let err = Product::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "A".to_owned(),
            "B".to_owned(),
            200,
            300,
            Some(100),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ValidationError::CreatedAfterDeleted {
                created_at: 200,
                deleted_at: 100,
            }
        );
    }
}
