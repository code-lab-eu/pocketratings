//! Password hashing (Argon2) and verification.
//!
//! Produces and verifies PHC-format strings suitable for storage in [`User`](crate::domain::user::User).

use argon2::password_hash::PasswordHash;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

/// Errors that can occur during password hashing or verification.
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    /// The stored hash string is not a valid PHC format (e.g. corrupted or wrong algorithm).
    #[error("invalid password hash format")]
    InvalidFormat,

    /// Hashing failed (e.g. parameter or length error).
    #[error("hashing failed")]
    Hashing,
}

/// Hash a plaintext password to a PHC string for storage.
///
/// Uses Argon2id with a random salt. Store the returned string in
/// [`User`](crate::domain::user::User)'s password field.
///
/// # Errors
///
/// Returns [`PasswordError::Hashing`] if hashing fails (e.g. password too long).
pub fn hash_password(plain: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|_| PasswordError::Hashing)?;
    Ok(hash.to_string())
}

/// Verify a plaintext password against a stored PHC hash.
///
/// Returns `Ok(true)` if the password matches, `Ok(false)` if it does not.
/// Returns `Err` only when the stored hash is invalid (e.g. malformed PHC string).
///
/// # Errors
///
/// Returns [`PasswordError::InvalidFormat`] if `stored_hash` is not a valid PHC string.
pub fn verify_password(plain: &str, stored_hash: &str) -> Result<bool, PasswordError> {
    let parsed = PasswordHash::new(stored_hash).map_err(|_| PasswordError::InvalidFormat)?;
    let ok = Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok();
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_succeeds() {
        let plain = "secret123";
        let hash = hash_password(plain).expect("hash should succeed");
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password(plain, &hash).expect("verify should succeed"));
    }

    #[test]
    fn wrong_password_fails_verification() {
        let hash = hash_password("correct").expect("hash should succeed");
        assert!(!verify_password("wrong", &hash).expect("verify should not error"));
    }

    #[test]
    fn invalid_hash_returns_error() {
        let err = verify_password("any", "not-a-phc-hash").unwrap_err();
        assert!(matches!(err, PasswordError::InvalidFormat));
    }

    #[test]
    fn different_salts_produce_different_hashes() {
        let h1 = hash_password("same").expect("hash 1");
        let h2 = hash_password("same").expect("hash 2");
        assert_ne!(h1, h2, "random salt should produce different hashes");
        assert!(verify_password("same", &h1).expect("verify 1"));
        assert!(verify_password("same", &h2).expect("verify 2"));
    }
}
