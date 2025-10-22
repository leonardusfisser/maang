use thiserror::Error;

/// Session management errors
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,

    #[error("Session expired")]
    Expired,

    #[error("Session invalid")]
    Invalid,

    #[error("Session creation failed: {0}")]
    CreationFailed(String),

    #[error("Session update failed: {0}")]
    UpdateFailed(String),

    #[error("Session deletion failed: {0}")]
    DeletionFailed(String),

    #[error("Session deserialization failed: {0}")]
    DeserializationFailed(String),

    #[error("Session serialization failed: {0}")]
    SerializationFailed(String),
}

impl SessionError {
    #[must_use]
    pub fn creation_failed(msg: impl Into<String>) -> Self {
        Self::CreationFailed(msg.into())
    }

    #[must_use]
    pub fn update_failed(msg: impl Into<String>) -> Self {
        Self::UpdateFailed(msg.into())
    }

    #[must_use]
    pub fn deletion_failed(msg: impl Into<String>) -> Self {
        Self::DeletionFailed(msg.into())
    }

    #[must_use]
    pub fn deserialization_failed(msg: impl Into<String>) -> Self {
        Self::DeserializationFailed(msg.into())
    }

    #[must_use]
    pub fn serialization_failed(msg: impl Into<String>) -> Self {
        Self::SerializationFailed(msg.into())
    }
}

impl From<serde_json::Error> for SessionError {
    fn from(err: serde_json::Error) -> Self {
        Self::DeserializationFailed(err.to_string())
    }
}