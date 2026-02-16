//! Location domain type with field validation.

use std::fmt;

use uuid::Uuid;

/// Validation errors for [`Location`] fields.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// The name field is empty.
    #[error("name must not be empty")]
    NameEmpty,
}

/// A validated location (store).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    id: Uuid,
    name: String,
    deleted_at: Option<i64>,
}

impl Location {
    /// Create a new `Location` after validating all fields.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if any field is invalid.
    pub fn new(id: Uuid, name: String, deleted_at: Option<i64>) -> Result<Self, ValidationError> {
        if name.trim().is_empty() {
            return Err(ValidationError::NameEmpty);
        }

        Ok(Self {
            id,
            name,
            deleted_at,
        })
    }

    /// Whether the location is active (not soft-deleted).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// The location's unique identifier.
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// The location name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// UNIX timestamp when the location was soft-deleted, if any.
    #[must_use]
    pub const fn deleted_at(&self) -> Option<i64> {
        self.deleted_at
    }
}

impl fmt::Display for Location {
    /// Format as `uuid (name)` for use in list and show.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.id(), self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_location(name: &str, deleted_at: Option<i64>) -> Result<Location, ValidationError> {
        Location::new(Uuid::new_v4(), name.to_owned(), deleted_at)
    }

    #[test]
    fn valid_location() {
        let loc = make_location("Supermarket", None);
        assert!(loc.is_ok());
        let loc = loc.unwrap();
        assert!(loc.is_active());
        assert_eq!(loc.name(), "Supermarket");
    }

    #[test]
    fn valid_deleted_location() {
        let loc = make_location("Old Store", Some(1_000));
        assert!(loc.is_ok());
        assert!(!loc.unwrap().is_active());
    }

    #[test]
    fn empty_name_is_rejected() {
        let err = make_location("", None).unwrap_err();
        assert_eq!(err, ValidationError::NameEmpty);
    }

    #[test]
    fn whitespace_only_name_is_rejected() {
        let err = make_location("   ", None).unwrap_err();
        assert_eq!(err, ValidationError::NameEmpty);
    }
}
