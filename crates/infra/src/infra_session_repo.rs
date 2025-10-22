use redis::aio::ConnectionManager;
use domain::Session;
use frame::InfraError;

/// Fetch a session from Redis by ID
pub async fn fetch_session_by_id(
    redis: &mut ConnectionManager,
    session_id: &str,
) -> Result<Option<Session>, InfraError> {
    let key = format!("session:{}", session_id);

    let value: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(redis)
        .await
        .map_err(|e| InfraError::cache(format!("Failed to get session: {e}")))?;

    match value {
        Some(json) => {
            let session: Session = serde_json::from_str(&json)
                .map_err(|e| InfraError::cache(format!("Failed to deserialize session: {e}")))?;
            Ok(Some(session))
        }
        None => Ok(None),
    }
}