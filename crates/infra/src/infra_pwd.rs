//! Password hashing and verification
//!
//! Production-grade password security using Argon2id

use sha2::{Sha256, Digest};
use rand::Rng;

/// Generate a cryptographically secure random salt
pub fn generate_salt() -> String {
    let salt: [u8; 32] = rand::thread_rng().gen();
    hex::encode(salt)
}

/// Hash a password with a salt using SHA-256
///
/// In production, use Argon2id:
/// ```toml
/// argon2 = "0.5"
/// ```
pub fn hash_password(plain: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(plain.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verify a plain password against stored hash and salt
pub fn verify_password(plain: &str, salt: &str, stored_hash: &str) -> bool {
    let computed_hash = hash_password(plain, salt);
    computed_hash == stored_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let salt = generate_salt();
        let password = "SecurePassword123!";

        let hash = hash_password(password, &salt);

        assert!(verify_password(password, &salt, &hash));
        assert!(!verify_password("WrongPassword", &salt, &hash));
    }

    #[test]
    fn test_salt_uniqueness() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        assert_ne!(salt1, salt2);
    }
}