use redis::RedisError;
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,
    #[error("Session expired")]
    Expired,
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Invalid session data")]
    Invalid,
}