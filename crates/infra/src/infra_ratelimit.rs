use frame::InfraError;
use redis::aio::ConnectionManager;

const RATELIMIT_PREFIX: &str = "ratelimit:";

/// Check and increment rate limit counter
pub async fn check_rate_limit(
    redis: &mut ConnectionManager,
    key: &str,
    max_requests: u32,
    window_secs: u64,
) -> Result<bool, InfraError> {
    let rate_key = format!("{RATELIMIT_PREFIX}{key}");

    // Get current count
    let count: Option<u32> = crate::infra_redis::get(redis, &rate_key)
        .await?
        .and_then(|v| v.parse().ok());

    let current = count.unwrap_or(0);

    if current >= max_requests {
        return Ok(false); // Rate limit exceeded
    }

    // Increment counter
    let new_count = current + 1;
    crate::infra_redis::set(redis, &rate_key, &new_count.to_string(), Some(window_secs))
        .await?;

    Ok(true) // Within rate limit
}

/// Reset rate limit counter
pub async fn reset_rate_limit(
    redis: &mut ConnectionManager,
    key: &str,
) -> Result<(), InfraError> {
    let rate_key = format!("{RATELIMIT_PREFIX}{key}");
    crate::infra_redis::del(redis, &rate_key).await?;
    Ok(())
}

/// Get current rate limit count
pub async fn get_rate_limit_count(
    redis: &mut ConnectionManager,
    key: &str,
) -> Result<u32, InfraError> {
    let rate_key = format!("{RATELIMIT_PREFIX}{key}");

    let count = crate::infra_redis::get(redis, &rate_key)
        .await?
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    Ok(count)
}