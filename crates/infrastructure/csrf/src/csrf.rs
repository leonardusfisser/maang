use sha2::{Sha256, Digest};
use rand::{Rng, distr::Alphanumeric};

#[derive(Debug)]
pub struct CsrfToken;

impl CsrfToken {
    pub fn generate(session_token: &str, secret: &str) -> String {
        let nonce: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let mut hasher = Sha256::new();
        hasher.update(session_token.as_bytes());
        hasher.update(secret.as_bytes());
        hasher.update(nonce.as_bytes());

        format!("{}:{}", nonce, hex::encode(hasher.finalize()))
    }

    pub fn validate(token: &str, session_token: &str, secret: &str) -> bool {
        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() != 2 {
            return false;
        }

        let nonce = parts[0];
        let expected_hash = parts[1];

        let mut hasher = Sha256::new();
        hasher.update(session_token.as_bytes());
        hasher.update(secret.as_bytes());
        hasher.update(nonce.as_bytes());

        hex::encode(hasher.finalize()) == expected_hash
    }
}