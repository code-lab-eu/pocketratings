//! Purchase domain type with field validation.

use std::fmt;

use rust_decimal::Decimal;
use uuid::Uuid;

/// Validation errors for [`Purchase`] fields.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// Quantity is less than 1.
    #[error("quantity must be at least 1 (got {quantity})")]
    QuantityInvalid {
        /// The invalid quantity value.
        quantity: i32,
    },

    /// Price is negative.
    #[error("price must not be negative (got {price})")]
    PriceInvalid {
        /// The invalid price value.
        price: Decimal,
    },
}

/// A validated purchase (user bought a product at a location).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purchase {
    id: Uuid,
    user_id: Uuid,
    product_id: Uuid,
    location_id: Uuid,
    quantity: i32,
    price: Decimal,
    purchased_at: i64,
    deleted_at: Option<i64>,
}

impl Purchase {
    /// Create a new `Purchase` after validating all fields.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if any field is invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        user_id: Uuid,
        product_id: Uuid,
        location_id: Uuid,
        quantity: i32,
        price: Decimal,
        purchased_at: i64,
        deleted_at: Option<i64>,
    ) -> Result<Self, ValidationError> {
        if quantity < 1 {
            return Err(ValidationError::QuantityInvalid { quantity });
        }
        if price < Decimal::ZERO {
            return Err(ValidationError::PriceInvalid { price });
        }

        Ok(Self {
            id,
            user_id,
            product_id,
            location_id,
            quantity,
            price,
            purchased_at,
            deleted_at,
        })
    }

    /// Whether the purchase is active (not soft-deleted).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// The purchase's unique identifier.
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// The user who made the purchase.
    #[must_use]
    pub const fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// The product that was purchased.
    #[must_use]
    pub const fn product_id(&self) -> Uuid {
        self.product_id
    }

    /// The location where the purchase was made.
    #[must_use]
    pub const fn location_id(&self) -> Uuid {
        self.location_id
    }

    /// Number of items.
    #[must_use]
    pub const fn quantity(&self) -> i32 {
        self.quantity
    }

    /// Unit price (e.g. in EUR).
    #[must_use]
    pub const fn price(&self) -> Decimal {
        self.price
    }

    /// UNIX timestamp when the purchase occurred.
    #[must_use]
    pub const fn purchased_at(&self) -> i64 {
        self.purchased_at
    }

    /// UNIX timestamp when the purchase was soft-deleted, if any.
    #[must_use]
    pub const fn deleted_at(&self) -> Option<i64> {
        self.deleted_at
    }
}

impl fmt::Display for Purchase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (product: {}, location: {}, qty: {}, price: {})",
            self.id(),
            self.product_id(),
            self.location_id(),
            self.quantity(),
            self.price()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_purchase(
        quantity: i32,
        price: Decimal,
        deleted_at: Option<i64>,
    ) -> Result<Purchase, ValidationError> {
        Purchase::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            quantity,
            price,
            1_000,
            deleted_at,
        )
    }

    #[test]
    fn valid_purchase() {
        let price: Decimal = "9.99".parse().expect("decimal");
        let p = make_purchase(1, price, None);
        assert!(p.is_ok());
        let p = p.unwrap();
        assert!(p.is_active());
        assert_eq!(p.quantity(), 1);
        assert_eq!(p.price(), price);
    }

    #[test]
    fn valid_deleted_purchase() {
        let p = make_purchase(2, Decimal::from(5), Some(1_000));
        assert!(p.is_ok());
        assert!(!p.unwrap().is_active());
    }

    #[test]
    fn quantity_zero_is_rejected() {
        let err = make_purchase(0, Decimal::from(1), None).unwrap_err();
        assert_eq!(err, ValidationError::QuantityInvalid { quantity: 0 });
    }

    #[test]
    fn quantity_negative_is_rejected() {
        let err = make_purchase(-1, Decimal::from(1), None).unwrap_err();
        assert_eq!(err, ValidationError::QuantityInvalid { quantity: -1 });
    }

    #[test]
    fn price_negative_is_rejected() {
        let err = make_purchase(1, Decimal::from(-1), None).unwrap_err();
        assert!(matches!(err, ValidationError::PriceInvalid { .. }));
    }

    #[test]
    fn price_zero_accepted() {
        assert!(make_purchase(1, Decimal::ZERO, None).is_ok());
    }
}
