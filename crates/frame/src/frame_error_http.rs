use thiserror::Error;

/// HTTP-specific errors that map to status codes
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Too many requests")]
    TooManyRequests,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Service unavailable")]
    ServiceUnavailable,
}

impl HttpError {
    #[must_use]
    pub const fn status_code(&self) -> u16 {
        match self {
            Self::BadRequest(_) => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound(_) => 404,
            Self::Conflict(_) => 409,
            Self::UnprocessableEntity(_) => 422,
            Self::TooManyRequests => 429,
            Self::InternalServerError => 500,
            Self::ServiceUnavailable => 503,
        }
    }

    #[must_use]
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    #[must_use]
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    #[must_use]
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    #[must_use]
    pub fn unprocessable(msg: impl Into<String>) -> Self {
        Self::UnprocessableEntity(msg.into())
    }
}