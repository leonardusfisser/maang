use thiserror::Error;

/// Infrastructure-level errors
#[derive(Debug, Error)]
pub enum InfraError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

impl InfraError {
    #[must_use]
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    #[must_use]
    pub fn cache(msg: impl Into<String>) -> Self {
        Self::Cache(msg.into())
    }

    #[must_use]
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    #[must_use]
    pub fn pool(msg: impl Into<String>) -> Self {
        Self::Pool(msg.into())
    }

    #[must_use]
    pub fn query(msg: impl Into<String>) -> Self {
        Self::Query(msg.into())
    }

    #[must_use]
    pub fn transaction(msg: impl Into<String>) -> Self {
        Self::Transaction(msg.into())
    }

    #[must_use]
    pub fn migration(msg: impl Into<String>) -> Self {
        Self::Migration(msg.into())
    }

    #[must_use]
    pub fn external_service(msg: impl Into<String>) -> Self {
        Self::ExternalService(msg.into())
    }
}

impl From<tokio_postgres::Error> for InfraError {
    fn from(err: tokio_postgres::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<deadpool_postgres::PoolError> for InfraError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        Self::Pool(err.to_string())
    }
}

impl From<redis::RedisError> for InfraError {
    fn from(err: redis::RedisError) -> Self {
        Self::Cache(err.to_string())
    }
}