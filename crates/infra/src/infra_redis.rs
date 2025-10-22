use frame::{AppError, InfraError, RedisConfig};
use redis::{aio::ConnectionManager, Client};

/// Create Redis connection manager
pub async fn create_redis_pool(config: &RedisConfig) -> Result<ConnectionManager, AppError> {
    let client = Client::open(config.url.as_str())
        .map_err(|e| AppError::init(format!("Failed to create Redis client: {e}")))?;

    let manager = ConnectionManager::new(client)
        .await
        .map_err(|e| AppError::init(format!("Failed to create Redis manager: {e}")))?;

    // Verify connection
    let _: String = redis::cmd("PING")
        .query_async(&mut manager.clone())
        .await
        .map_err(|e| AppError::init(format!("Redis health check failed: {e}")))?;

    tracing::info!("Redis connection manager created successfully");
    Ok(manager)
}

/// Set a key-value pair with optional TTL
pub async fn set(
    manager: &mut ConnectionManager,
    key: &str,
    value: &str,
    ttl_secs: Option<u64>,
) -> Result<(), InfraError> {
    if let Some(ttl) = ttl_secs {
        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl)
            .arg(value)
            .query_async(manager)
            .await
            .map_err(InfraError::from)
    } else {
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query_async(manager)
            .await
            .map_err(InfraError::from)
    }
}

/// Get a value by key
pub async fn get(
    manager: &mut ConnectionManager,
    key: &str,
) -> Result<Option<String>, InfraError> {
    redis::cmd("GET")
        .arg(key)
        .query_async(manager)
        .await
        .map_err(InfraError::from)
}

/// Delete a key
pub async fn del(manager: &mut ConnectionManager, key: &str) -> Result<(), InfraError> {
    redis::cmd("DEL")
        .arg(key)
        .query_async(manager)
        .await
        .map_err(InfraError::from)
}

/// Check if key exists
pub async fn exists(manager: &mut ConnectionManager, key: &str) -> Result<bool, InfraError> {
    let result: i32 = redis::cmd("EXISTS")
        .arg(key)
        .query_async(manager)
        .await
        .map_err(InfraError::from)?;

    Ok(result == 1)
}

/// Set TTL on existing key
pub async fn expire(
    manager: &mut ConnectionManager,
    key: &str,
    ttl_secs: u64,
) -> Result<(), InfraError> {
    redis::cmd("EXPIRE")
        .arg(key)
        .arg(ttl_secs)
        .query_async(manager)
        .await
        .map_err(InfraError::from)
}