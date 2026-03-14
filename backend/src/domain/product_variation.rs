//! Product variation domain type with field validation.

use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Valid unit values for a product variation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Grams,
    Milliliters,
    Other,
    None,
}

impl Unit {
    /// All valid units in canonical string form.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [Self::Grams, Self::Milliliters, Self::Other, Self::None]
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Grams => "grams",
            Self::Milliliters => "milliliters",
            Self::Other => "other",
            Self::None => "none",
        })
    }
}

impl FromStr for Unit {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "grams" => Ok(Self::Grams),
            "milliliters" => Ok(Self::Milliliters),
            "other" => Ok(Self::Other),
            "none" => Ok(Self::None),
            other => Err(ValidationError::UnitInvalid {
                unit: other.to_string(),
            }),
        }
    }
}

/// Validation errors for [`ProductVariation`] fields.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// The unit is not one of the allowed values.
    #[error("unit must be one of: grams, milliliters, other, none (got {unit:?})")]
    UnitInvalid { unit: String },
}

/// A validated product variation (e.g. size or unit; belongs to a product).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductVariation {
    id: Uuid,
    product_id: Uuid,
    label: String,
    unit: String,
    quantity: Option<u32>,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
}

impl ProductVariation {
    /// Create a new `ProductVariation` after validating fields.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if `unit` is not one of grams, milliliters, other, none.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        product_id: Uuid,
        label: &str,
        unit: &str,
        quantity: Option<u32>,
        created_at: i64,
        updated_at: i64,
        deleted_at: Option<i64>,
    ) -> Result<Self, ValidationError> {
        let unit_val = Unit::from_str(unit)?;
        Ok(Self {
            id,
            product_id,
            label: label.trim().to_string(),
            unit: unit_val.to_string(),
            quantity,
            created_at,
            updated_at,
            deleted_at,
        })
    }

    /// Whether the variation is active (not soft-deleted).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub const fn product_id(&self) -> Uuid {
        self.product_id
    }

    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[must_use]
    pub fn unit(&self) -> &str {
        &self.unit
    }

    #[must_use]
    pub const fn quantity(&self) -> Option<u32> {
        self.quantity
    }

    #[must_use]
    pub const fn created_at(&self) -> i64 {
        self.created_at
    }

    #[must_use]
    pub const fn updated_at(&self) -> i64 {
        self.updated_at
    }

    #[must_use]
    pub const fn deleted_at(&self) -> Option<i64> {
        self.deleted_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_variation_grams() {
        let v = ProductVariation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "500 g",
            "grams",
            Some(500),
            1_000,
            1_000,
            None,
        );
        assert!(v.is_ok());
        let v = v.unwrap();
        assert_eq!(v.label(), "500 g");
        assert_eq!(v.unit(), "grams");
        assert_eq!(v.quantity(), Some(500));
    }

    #[test]
    fn valid_variation_none() {
        let v = ProductVariation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "",
            "none",
            None,
            1_000,
            1_000,
            None,
        );
        assert!(v.is_ok());
        assert_eq!(v.as_ref().unwrap().quantity(), None);
    }

    #[test]
    fn invalid_unit_rejected() {
        let v = ProductVariation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "",
            "kilograms",
            None,
            1_000,
            1_000,
            None,
        );
        assert!(matches!(v, Err(ValidationError::UnitInvalid { .. })));
    }
}
