use frame::CsrfError;
use rand::Rng;
use sha2::{Digest, Sha256};

const CSRF_PREFIX: &str = "csrf:";

/// Generate a CSRF token
pub fn generate_token() -> Result<String, CsrfError> {
    let random_bytes: [u8; 32] = rand::rng().random();
    let mut hasher = Sha256::new();
    hasher.update(random_bytes);
    Ok(hex::encode(hasher.finalize()))
}

/// Store CSRF token in Redis
pub async fn store_token(
    redis: &mut redis::aio::ConnectionManager,
    session_id: &str,
    token: &str,
    ttl_secs: u64,
) -> Result<(), CsrfError> {
    let key = format!("{CSRF_PREFIX}{session_id}");

    crate::infra_redis::set(redis, &key, token, Some(ttl_secs))
        .await
        .map_err(|e| CsrfError::generation_failed(e.to_string()))?;

    Ok(())
}

/// Validate CSRF token
pub async fn validate_token(
    redis: &mut redis::aio::ConnectionManager,
    session_id: &str,
    token: &str,
) -> Result<(), CsrfError> {
    let key = format!("{CSRF_PREFIX}{session_id}");

    let stored_token = crate::infra_redis::get(redis, &key)
        .await
        .map_err(|e| CsrfError::validation_failed(e.to_string()))?
        .ok_or(CsrfError::TokenMissing)?;

    if stored_token != token {
        return Err(CsrfError::TokenInvalid);
    }

    Ok(())
}

/// Delete CSRF token
pub async fn delete_token(
    redis: &mut redis::aio::ConnectionManager,
    session_id: &str,
) -> Result<(), CsrfError> {
    let key = format!("{CSRF_PREFIX}{session_id}");

    crate::infra_redis::del(redis, &key)
        .await
        .map_err(|e| CsrfError::validation_failed(e.to_string()))?;

    Ok(())
}