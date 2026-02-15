//! User domain type with field validation.

use uuid::Uuid;

/// Validation errors for [`User`] fields.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// The name field is empty.
    #[error("name must not be empty")]
    NameEmpty,

    /// The email address is not structurally valid.
    #[error("invalid email address: {0}")]
    EmailInvalid(String),

    /// The password hash is empty.
    #[error("password hash must not be empty")]
    PasswordEmpty,

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

/// A validated user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
    password: String,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
}

impl User {
    /// Create a new `User` after validating all fields.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if any field is invalid.
    pub fn new(
        id: Uuid,
        name: String,
        email: String,
        password: String,
        created_at: i64,
        updated_at: i64,
        deleted_at: Option<i64>,
    ) -> Result<Self, ValidationError> {
        if name.trim().is_empty() {
            return Err(ValidationError::NameEmpty);
        }

        if !is_valid_email(&email) {
            return Err(ValidationError::EmailInvalid(email));
        }

        if password.is_empty() {
            return Err(ValidationError::PasswordEmpty);
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
            name,
            email,
            password,
            created_at,
            updated_at,
            deleted_at,
        })
    }

    /// Whether the user account is active (not soft-deleted).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// The user's unique identifier.
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// The user's display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The user's email address.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    /// The stored password hash (Argon2 PHC string).
    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    /// UNIX timestamp when the user was created.
    #[must_use]
    pub const fn created_at(&self) -> i64 {
        self.created_at
    }

    /// UNIX timestamp when the user was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> i64 {
        self.updated_at
    }

    /// UNIX timestamp when the user was soft-deleted, if any.
    #[must_use]
    pub const fn deleted_at(&self) -> Option<i64> {
        self.deleted_at
    }

    /// Verify that a plaintext password matches this user's stored hash.
    ///
    /// Delegates to [`crate::auth::password::verify_password`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::auth::password::PasswordError`] if the stored hash is invalid (e.g. malformed).
    pub fn verify_password(
        &self,
        plain: &str,
    ) -> Result<bool, crate::auth::password::PasswordError> {
        crate::auth::password::verify_password(plain, self.password())
    }
}

/// Basic structural validation for an email address.
///
/// Checks that there is exactly one `@`, a non-empty local part, and a
/// domain part that contains at least one `.` with no empty labels.
fn is_valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };

    if local.is_empty() || domain.is_empty() {
        return false;
    }

    // Reject additional @ signs.
    if domain.contains('@') {
        return false;
    }

    // Domain must have at least one dot, and no empty labels.
    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() < 2 {
        return false;
    }
    labels.iter().all(|l| !l.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a valid user, returning the Result so tests can
    /// assert on the outcome.
    fn make_user(
        name: &str,
        email: &str,
        password: &str,
        created_at: i64,
        updated_at: i64,
        deleted_at: Option<i64>,
    ) -> Result<User, ValidationError> {
        User::new(
            Uuid::new_v4(),
            name.to_owned(),
            email.to_owned(),
            password.to_owned(),
            created_at,
            updated_at,
            deleted_at,
        )
    }

    // -- happy path --

    #[test]
    fn valid_user() {
        let user = make_user(
            "Alice",
            "alice@example.com",
            "$argon2id$hash",
            1_000,
            1_000,
            None,
        );
        assert!(user.is_ok());
        let user = user.unwrap();
        assert!(user.is_active());
        assert_eq!(user.name(), "Alice");
        assert_eq!(user.email(), "alice@example.com");
    }

    #[test]
    fn valid_deleted_user() {
        let user = make_user(
            "Bob",
            "bob@example.com",
            "$argon2id$hash",
            1_000,
            2_000,
            Some(3_000),
        );
        assert!(user.is_ok());
        assert!(!user.unwrap().is_active());
    }

    // -- name validation --

    #[test]
    fn empty_name_is_rejected() {
        let err = make_user("", "a@b.com", "hash", 1, 1, None).unwrap_err();
        assert_eq!(err, ValidationError::NameEmpty);
    }

    #[test]
    fn whitespace_only_name_is_rejected() {
        let err = make_user("   ", "a@b.com", "hash", 1, 1, None).unwrap_err();
        assert_eq!(err, ValidationError::NameEmpty);
    }

    // -- email validation --

    #[test]
    fn email_without_at_is_rejected() {
        let err = make_user("A", "noatsign", "hash", 1, 1, None).unwrap_err();
        assert!(matches!(err, ValidationError::EmailInvalid(_)));
    }

    #[test]
    fn email_without_domain_dot_is_rejected() {
        let err = make_user("A", "a@localhost", "hash", 1, 1, None).unwrap_err();
        assert!(matches!(err, ValidationError::EmailInvalid(_)));
    }

    #[test]
    fn email_with_empty_local_is_rejected() {
        let err = make_user("A", "@example.com", "hash", 1, 1, None).unwrap_err();
        assert!(matches!(err, ValidationError::EmailInvalid(_)));
    }

    #[test]
    fn email_with_double_at_is_rejected() {
        let err = make_user("A", "a@@b.com", "hash", 1, 1, None).unwrap_err();
        assert!(matches!(err, ValidationError::EmailInvalid(_)));
    }

    #[test]
    fn email_with_empty_domain_label_is_rejected() {
        let err = make_user("A", "a@.com", "hash", 1, 1, None).unwrap_err();
        assert!(matches!(err, ValidationError::EmailInvalid(_)));
    }

    // -- password validation --

    #[test]
    fn empty_password_is_rejected() {
        let err = make_user("A", "a@b.com", "", 1, 1, None).unwrap_err();
        assert_eq!(err, ValidationError::PasswordEmpty);
    }

    // -- timestamp validation --

    #[test]
    fn created_after_updated_is_rejected() {
        let err = make_user("A", "a@b.com", "hash", 200, 100, None).unwrap_err();
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
        let err = make_user("A", "a@b.com", "hash", 200, 300, Some(100)).unwrap_err();
        assert_eq!(
            err,
            ValidationError::CreatedAfterDeleted {
                created_at: 200,
                deleted_at: 100,
            }
        );
    }

    #[test]
    fn created_equal_to_updated_is_ok() {
        assert!(make_user("A", "a@b.com", "hash", 100, 100, None).is_ok());
    }

    #[test]
    fn created_equal_to_deleted_is_ok() {
        assert!(make_user("A", "a@b.com", "hash", 100, 100, Some(100)).is_ok());
    }

    // -- email helper unit tests --

    #[test]
    fn valid_emails() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("first.last@sub.domain.org"));
        assert!(is_valid_email("x@y.co"));
    }

    #[test]
    fn invalid_emails() {
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("noat"));
        assert!(!is_valid_email("@missing-local.com"));
        assert!(!is_valid_email("missing-domain@"));
        assert!(!is_valid_email("two@@ats.com"));
        assert!(!is_valid_email("no-dot@localhost"));
        assert!(!is_valid_email("trailing@dot."));
    }

    // -- verify_password (delegates to auth::password) --

    #[test]
    fn verify_password_correct_returns_true() {
        let hash = crate::auth::password::hash_password("secret").expect("hash");
        let user = make_user("U", "u@x.co", &hash, 1, 1, None).unwrap();
        assert!(user.verify_password("secret").expect("verify"));
    }

    #[test]
    fn verify_password_wrong_returns_false() {
        let hash = crate::auth::password::hash_password("secret").expect("hash");
        let user = make_user("U", "u@x.co", &hash, 1, 1, None).unwrap();
        assert!(!user.verify_password("wrong").expect("verify"));
    }
}
