//! JWT issue and verify for API auth.

use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims: subject (user id) and expiration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — user id (UUID string).
    pub sub: String,
    /// Expiration time (Unix timestamp, seconds).
    pub exp: u64,
}

/// Errors from JWT encoding or decoding.
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("JWT encoding failed")]
    Encode,

    #[error("JWT invalid or expired")]
    Invalid,
}

/// Issue a new JWT for the given user with the given expiration.
///
/// # Errors
///
/// Returns [`JwtError::Encode`] if encoding fails.
pub fn issue_token(secret: &str, user_id: Uuid, expiration_secs: u64) -> Result<String, JwtError> {
    let now = get_current_timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + expiration_secs,
    };
    let key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &key).map_err(|_| JwtError::Encode)
}

/// Verify a JWT and return its claims.
///
/// # Errors
///
/// Returns [`JwtError::Invalid`] if the token is missing, malformed, expired, or signature is wrong.
pub fn verify_token(secret: &str, token: &str) -> Result<Claims, JwtError> {
    let key = DecodingKey::from_secret(secret.as_ref());
    let mut validation = Validation::default();
    validation.leeway = 0;
    let token_data = decode::<Claims>(token, &key, &validation).map_err(|_| JwtError::Invalid)?;
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn issue_then_verify_succeeds() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let exp_secs = 3600u64;
        let token = issue_token(secret, user_id, exp_secs).expect("issue should succeed");
        let claims = verify_token(secret, &token).expect("verify should succeed");
        assert_eq!(claims.sub, user_id.to_string());
        assert!(claims.exp > get_current_timestamp());
    }

    #[test]
    fn verify_with_wrong_secret_fails() {
        let token = issue_token("secret-a", Uuid::new_v4(), 3600).expect("issue");
        let err = verify_token("secret-b", &token).unwrap_err();
        assert!(matches!(err, JwtError::Invalid));
    }

    #[test]
    fn verify_expired_token_fails() {
        let secret = "test";
        let user_id = Uuid::new_v4();
        let now = get_current_timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now.saturating_sub(1),
        };
        let key = EncodingKey::from_secret(secret.as_ref());
        let token = encode(&Header::default(), &claims, &key).expect("encode");
        let err = verify_token(secret, &token).unwrap_err();
        assert!(matches!(err, JwtError::Invalid));
    }

    #[test]
    fn verify_garbage_fails() {
        let err = verify_token("secret", "not.a.valid.jwt").unwrap_err();
        assert!(matches!(err, JwtError::Invalid));
    }
}
