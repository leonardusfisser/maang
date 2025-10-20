#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Email error: {0}")]
    Email(String),

    #[error("Payment error: {0}")]
    Payment(String),

    #[error("Queue error: {0}")]
    Queue(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

impl InfraError {
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    pub fn cache(msg: impl Into<String>) -> Self {
        Self::Cache(msg.into())
    }

    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }

    pub fn email(msg: impl Into<String>) -> Self {
        Self::Email(msg.into())
    }

    pub fn payment(msg: impl Into<String>) -> Self {
        Self::Payment(msg.into())
    }
}

impl From<serde_json::Error> for InfraError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for InfraError {
    fn from(err: std::io::Error) -> Self {
        Self::Network(err.to_string())
    }
}