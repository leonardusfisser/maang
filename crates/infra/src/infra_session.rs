use frame::SessionError;
use domain::Session;
use redis::aio::ConnectionManager;

const SESSION_PREFIX: &str = "session:";

/// Store session in Redis
pub async fn store_session(
    redis: &mut ConnectionManager,
    session: &Session,
) -> Result<(), SessionError> {
    let key = format!("{SESSION_PREFIX}{}", session.id);
    let value = serde_json::to_string(session)
        .map_err(|e| SessionError::serialization_failed(e.to_string()))?;

    let ttl = session.expires_at - session.created_at;
    let ttl_u64 = u64::try_from(ttl).unwrap_or(3600);

    crate::infra_redis::set(redis, &key, &value, Some(ttl_u64))
        .await
        .map_err(|e| SessionError::creation_failed(e.to_string()))?;

    Ok(())
}

/// Retrieve session from Redis
pub async fn get_session(
    redis: &mut ConnectionManager,
    session_id: &str,
) -> Result<Session, SessionError> {
    let key = format!("{SESSION_PREFIX}{session_id}");

    let value = crate::infra_redis::get(redis, &key)
        .await
        .map_err(|e| SessionError::deserialization_failed(e.to_string()))?
        .ok_or(SessionError::NotFound)?;

    let session: Session = serde_json::from_str(&value)
        .map_err(|e| SessionError::deserialization_failed(e.to_string()))?;

    if session.is_expired() {
        let _ = delete_session(redis, session_id).await;
        return Err(SessionError::Expired);
    }

    Ok(session)
}

/// Delete session from Redis
pub async fn delete_session(
    redis: &mut ConnectionManager,
    session_id: &str,
) -> Result<(), SessionError> {
    let key = format!("{SESSION_PREFIX}{session_id}");

    crate::infra_redis::del(redis, &key)
        .await
        .map_err(|e| SessionError::deletion_failed(e.to_string()))?;

    Ok(())
}

/// Update session activity timestamp
pub async fn touch_session(
    redis: &mut ConnectionManager,
    session_id: &str,
) -> Result<(), SessionError> {
    let mut session = get_session(redis, session_id).await?;
    session.touch();
    store_session(redis, &session).await?;
    Ok(())
}