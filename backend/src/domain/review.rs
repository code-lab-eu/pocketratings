//! Review domain type with field validation.

use std::fmt;

use rust_decimal::Decimal;
use uuid::Uuid;

/// Validation errors for [`Review`] fields.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// Rating is outside the valid range 1–5.
    #[error("rating must be between 1 and 5 (got {rating})")]
    RatingOutOfRange {
        /// The invalid rating value.
        rating: Decimal,
    },

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

/// A validated review (user's rating and optional text for a product).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Review {
    id: Uuid,
    product_id: Uuid,
    user_id: Uuid,
    rating: Decimal,
    text: Option<String>,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
}

impl Review {
    /// Create a new `Review` after validating all fields.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if any field is invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        product_id: Uuid,
        user_id: Uuid,
        rating: Decimal,
        text: Option<String>,
        created_at: i64,
        updated_at: i64,
        deleted_at: Option<i64>,
    ) -> Result<Self, ValidationError> {
        let five = Decimal::from(5);
        if rating < Decimal::ONE || rating > five {
            return Err(ValidationError::RatingOutOfRange { rating });
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
            product_id,
            user_id,
            rating,
            text,
            created_at,
            updated_at,
            deleted_at,
        })
    }

    /// Whether the review is active (not soft-deleted).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// The review's unique identifier.
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// The product this review is for.
    #[must_use]
    pub const fn product_id(&self) -> Uuid {
        self.product_id
    }

    /// The user who wrote the review.
    #[must_use]
    pub const fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// The rating (1–5).
    #[must_use]
    pub const fn rating(&self) -> Decimal {
        self.rating
    }

    /// Optional review text.
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// UNIX timestamp when the review was created.
    #[must_use]
    pub const fn created_at(&self) -> i64 {
        self.created_at
    }

    /// UNIX timestamp when the review was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> i64 {
        self.updated_at
    }

    /// UNIX timestamp when the review was soft-deleted, if any.
    #[must_use]
    pub const fn deleted_at(&self) -> Option<i64> {
        self.deleted_at
    }
}

impl fmt::Display for Review {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (product: {}, user: {}, rating: {})",
            self.id(),
            self.product_id(),
            self.user_id(),
            self.rating()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_review(
        rating: Decimal,
        text: Option<&str>,
        deleted_at: Option<i64>,
    ) -> Result<Review, ValidationError> {
        Review::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            rating,
            text.map(str::to_owned),
            1_000,
            1_000,
            deleted_at,
        )
    }

    #[test]
    fn valid_review() {
        let rev = make_review(Decimal::from(4), None, None);
        assert!(rev.is_ok());
        let rev = rev.unwrap();
        assert!(rev.is_active());
        assert_eq!(rev.rating(), Decimal::from(4));
        assert_eq!(rev.text(), None);
    }

    #[test]
    fn valid_review_with_text() {
        let rev = make_review(Decimal::from(5), Some("Great!"), None);
        assert!(rev.is_ok());
        assert_eq!(rev.unwrap().text(), Some("Great!"));
    }

    #[test]
    fn valid_deleted_review() {
        let rev = make_review(Decimal::from(3), None, Some(1_000));
        assert!(rev.is_ok());
        assert!(!rev.unwrap().is_active());
    }

    #[test]
    fn rating_below_one_is_rejected() {
        let err = make_review(Decimal::ZERO, None, None).unwrap_err();
        assert!(matches!(err, ValidationError::RatingOutOfRange { .. }));
    }

    #[test]
    fn rating_above_five_is_rejected() {
        let err = make_review(Decimal::from(6), None, None).unwrap_err();
        assert!(matches!(err, ValidationError::RatingOutOfRange { .. }));
    }

    #[test]
    fn rating_one_and_five_accepted() {
        assert!(make_review(Decimal::ONE, None, None).is_ok());
        assert!(make_review(Decimal::from(5), None, None).is_ok());
    }

    #[test]
    fn decimal_rating_accepted() {
        assert!(make_review(Decimal::try_from(4.5).unwrap(), None, None).is_ok());
    }

    #[test]
    fn created_after_updated_is_rejected() {
        let err = Review::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Decimal::from(4),
            None,
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
        let err = Review::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Decimal::from(4),
            None,
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
