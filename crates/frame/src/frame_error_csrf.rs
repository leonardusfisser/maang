use thiserror::Error;

/// CSRF protection errors
#[derive(Debug, Error)]
pub enum CsrfError {
    #[error("CSRF token missing")]
    TokenMissing,

    #[error("CSRF token invalid")]
    TokenInvalid,

    #[error("CSRF token expired")]
    TokenExpired,

    #[error("CSRF token generation failed: {0}")]
    GenerationFailed(String),

    #[error("CSRF validation failed: {0}")]
    ValidationFailed(String),
}

impl CsrfError {
    #[must_use]
    pub fn generation_failed(msg: impl Into<String>) -> Self {
        Self::GenerationFailed(msg.into())
    }

    #[must_use]
    pub fn validation_failed(msg: impl Into<String>) -> Self {
        Self::ValidationFailed(msg.into())
    }
}