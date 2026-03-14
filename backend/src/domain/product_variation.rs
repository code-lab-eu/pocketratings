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
    unit: Unit,
    quantity: Option<u32>,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
}

impl ProductVariation {
    /// Default display label when label is empty and unit+quantity allow it.
    /// Grams: 1000+ -> "N kg", else "N g". Milliliters: 1000+ -> "N l", 200+ and divisible by 10 -> "N cl", else "N ml".
    #[must_use]
    pub fn default_label_for_quantity(unit: Unit, quantity: Option<u32>) -> Option<String> {
        let q = quantity?;
        Some(match unit {
            Unit::Grams => {
                if q >= 1000 {
                    let n = f64::from(q) / 1000.0;
                    format!("{n} kg")
                } else {
                    format!("{q} g")
                }
            }
            Unit::Milliliters => {
                if q >= 1000 {
                    let n = f64::from(q) / 1000.0;
                    format!("{n} l")
                } else if q >= 200 && q % 10 == 0 {
                    format!("{} cl", q / 10)
                } else {
                    format!("{q} ml")
                }
            }
            Unit::Other | Unit::None => return None,
        })
    }

    /// Create a new `ProductVariation` after validating fields.
    /// When label is empty and unit is grams or milliliters with a quantity, the label
    /// is auto-generated (e.g. 1000 grams -> "1 kg", 750 milliliters -> "75 cl").
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
        let label = if label.trim().is_empty() {
            Self::default_label_for_quantity(unit_val, quantity).unwrap_or_default()
        } else {
            label.trim().to_string()
        };
        Ok(Self {
            id,
            product_id,
            label,
            unit: unit_val,
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
    pub const fn unit(&self) -> Unit {
        self.unit
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
        assert_eq!(v.unit(), Unit::Grams);
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

    #[test]
    fn default_label_grams_kg() {
        assert_eq!(
            ProductVariation::default_label_for_quantity(Unit::Grams, Some(1000)),
            Some("1 kg".to_string())
        );
        assert_eq!(
            ProductVariation::default_label_for_quantity(Unit::Grams, Some(2500)),
            Some("2.5 kg".to_string())
        );
    }

    #[test]
    fn default_label_grams_g() {
        assert_eq!(
            ProductVariation::default_label_for_quantity(Unit::Grams, Some(500)),
            Some("500 g".to_string())
        );
    }

    #[test]
    fn default_label_milliliters_l_cl_ml() {
        assert_eq!(
            ProductVariation::default_label_for_quantity(Unit::Milliliters, Some(1000)),
            Some("1 l".to_string())
        );
        assert_eq!(
            ProductVariation::default_label_for_quantity(Unit::Milliliters, Some(750)),
            Some("75 cl".to_string())
        );
        assert_eq!(
            ProductVariation::default_label_for_quantity(Unit::Milliliters, Some(500)),
            Some("50 cl".to_string())
        );
        assert_eq!(
            ProductVariation::default_label_for_quantity(Unit::Milliliters, Some(100)),
            Some("100 ml".to_string())
        );
    }

    #[test]
    fn new_with_empty_label_and_quantity_auto_fills_grams() {
        let v = ProductVariation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "",
            "grams",
            Some(1000),
            1_000,
            1_000,
            None,
        );
        assert!(v.is_ok());
        assert_eq!(v.unwrap().label(), "1 kg");
    }

    #[test]
    fn new_with_empty_label_and_quantity_auto_fills_milliliters() {
        let v = ProductVariation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "",
            "milliliters",
            Some(750),
            1_000,
            1_000,
            None,
        );
        assert!(v.is_ok());
        assert_eq!(v.unwrap().label(), "75 cl");
    }
}
