use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Generate a secure random ID using UUID v4
#[must_use]
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Hash a password using SHA-256 with salt
#[must_use]
pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verify a password against a hash
#[must_use]
pub fn verify_password(password: &str, salt: &str, hash: &str) -> bool {
    let computed = hash_password(password, salt);
    computed == hash
}

/// Generate a random salt
#[must_use]
pub fn generate_salt() -> String {
    use rand::Rng;
    let salt: [u8; 32] = rand::rng().random();
    hex::encode(salt)
}

/// Get current timestamp in seconds
#[must_use]
pub fn current_timestamp() -> i64 {
    use time::OffsetDateTime;
    OffsetDateTime::now_utc().unix_timestamp()
}

/// Format timestamp for display
pub fn format_timestamp(timestamp: i64) -> Result<String, time::error::Format> {
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};

    let datetime = OffsetDateTime::from_unix_timestamp(timestamp)
        .map_err(|_| time::error::Format::InvalidComponent("timestamp"))?;

    datetime.format(&Rfc3339)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id();
        assert_eq!(id.len(), 36);
    }

    #[test]
    fn test_password_hashing() {
        let password = "test_password";
        let salt = "test_salt";
        let hash = hash_password(password, salt);

        assert!(verify_password(password, salt, &hash));
        assert!(!verify_password("wrong", salt, &hash));
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 0);
    }
}