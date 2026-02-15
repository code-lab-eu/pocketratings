//! Category domain type with field validation.

use uuid::Uuid;

/// Validation errors for [`Category`] fields.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
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

/// A validated category (optionally under a parent for hierarchy).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    id: Uuid,
    parent_id: Option<Uuid>,
    name: String,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
}

impl Category {
    /// Create a new `Category` after validating all fields.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if any field is invalid.
    pub fn new(
        id: Uuid,
        parent_id: Option<Uuid>,
        name: String,
        created_at: i64,
        updated_at: i64,
        deleted_at: Option<i64>,
    ) -> Result<Self, ValidationError> {
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
            parent_id,
            name,
            created_at,
            updated_at,
            deleted_at,
        })
    }

    /// Whether the category is active (not soft-deleted).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// The category's unique identifier.
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Parent category id, if this is a subcategory.
    #[must_use]
    pub const fn parent_id(&self) -> Option<Uuid> {
        self.parent_id
    }

    /// The category name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// UNIX timestamp when the category was created.
    #[must_use]
    pub const fn created_at(&self) -> i64 {
        self.created_at
    }

    /// UNIX timestamp when the category was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> i64 {
        self.updated_at
    }

    /// UNIX timestamp when the category was soft-deleted, if any.
    #[must_use]
    pub const fn deleted_at(&self) -> Option<i64> {
        self.deleted_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_category(
        name: &str,
        parent_id: Option<Uuid>,
        deleted_at: Option<i64>,
    ) -> Result<Category, ValidationError> {
        Category::new(
            Uuid::new_v4(),
            parent_id,
            name.to_owned(),
            1_000,
            1_000,
            deleted_at,
        )
    }

    #[test]
    fn valid_root_category() {
        let cat = make_category("Electronics", None, None);
        assert!(cat.is_ok());
        let cat = cat.unwrap();
        assert!(cat.is_active());
        assert_eq!(cat.name(), "Electronics");
        assert_eq!(cat.parent_id(), None);
    }

    #[test]
    fn valid_subcategory() {
        let parent_id = Uuid::new_v4();
        let cat = make_category("Phones", Some(parent_id), None);
        assert!(cat.is_ok());
        assert_eq!(cat.as_ref().unwrap().parent_id(), Some(parent_id));
    }

    #[test]
    fn valid_deleted_category() {
        let cat = make_category("Old", None, Some(1_000));
        assert!(cat.is_ok());
        assert!(!cat.unwrap().is_active());
    }

    #[test]
    fn empty_name_is_rejected() {
        let err = make_category("", None, None).unwrap_err();
        assert_eq!(err, ValidationError::NameEmpty);
    }

    #[test]
    fn whitespace_only_name_is_rejected() {
        let err = make_category("   ", None, None).unwrap_err();
        assert_eq!(err, ValidationError::NameEmpty);
    }

    #[test]
    fn created_after_updated_is_rejected() {
        let err = Category::new(
            Uuid::new_v4(),
            None,
            "A".to_owned(),
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
        let err = Category::new(
            Uuid::new_v4(),
            None,
            "A".to_owned(),
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
