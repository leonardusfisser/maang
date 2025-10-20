use crate::{DomainError, HttpError, InfraError, SessionError, SecurityError};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Infrastructure error: {0}")]
    Infra(#[from] InfraError),

    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),

    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;

        match self {
            Self::Domain(_) => StatusCode::BAD_REQUEST,
            Self::Infra(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Http(e) => e.status_code(),
            Self::Session(_) => StatusCode::UNAUTHORIZED,
            Self::Security(_) => StatusCode::FORBIDDEN,
            Self::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code())
            .json(serde_json::json!({
                "error": self.to_string()
            }))
    }
}