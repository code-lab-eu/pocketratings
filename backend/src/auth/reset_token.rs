//! Password reset tokens: generation and hashing.
//!
//! The raw token travels in the reset link; only its SHA-256 digest is stored, so a leaked
//! database cannot be turned back into working links. Argon2 (used for passwords) is unsuitable
//! here because looking a token up requires a deterministic hash.

use sha2::{Digest, Sha256};

/// How long a reset link stays valid: 12 hours, in seconds.
pub const TOKEN_TTL_SECONDS: i64 = 12 * 3600;

/// Number of random bytes in a token; hex-encoded this yields a 64-character string.
const TOKEN_BYTES: usize = 32;

/// Generate a new random reset token (hex-encoded, 64 characters).
///
/// # Errors
///
/// Returns an error if the operating system's random number generator is unavailable.
pub fn generate() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes)?;
    Ok(hex::encode(bytes))
}

/// Hash a raw token for storage and look-up (SHA-256, hex-encoded).
#[must_use]
pub fn hash(raw: &str) -> String {
    hex::encode(Sha256::digest(raw.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::{TOKEN_TTL_SECONDS, generate, hash};

    #[test]
    fn generate_returns_64_hex_characters() {
        let token = generate().expect("generate");
        assert_eq!(token.len(), 64);
        assert!(
            token.chars().all(|c| c.is_ascii_hexdigit()),
            "token should be hex: {token}"
        );
    }

    #[test]
    fn generate_returns_a_different_token_each_call() {
        let first = generate().expect("generate");
        let second = generate().expect("generate");
        assert_ne!(first, second);
    }

    #[test]
    fn hash_is_deterministic_and_differs_per_token() {
        let token = generate().expect("generate");
        assert_eq!(hash(&token), hash(&token));
        assert_ne!(hash(&token), hash(&generate().expect("generate")));
    }

    #[test]
    fn hash_returns_64_hex_characters() {
        let digest = hash("some-token");
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_matches_known_sha256_vector() {
        assert_eq!(
            hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn token_ttl_is_twelve_hours() {
        assert_eq!(TOKEN_TTL_SECONDS, 43_200);
    }
}
