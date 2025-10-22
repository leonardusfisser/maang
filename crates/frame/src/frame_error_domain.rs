use thiserror::Error;

/// Domain/business logic errors
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation failed: {field}: {message}")]
    Validation { field: String, message: String },

    #[error("Entity not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },

    #[error("Duplicate entity: {entity} with {field} = {value}")]
    Duplicate {
        entity: String,
        field: String,
        value: String,
    },

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Operation not allowed: {0}")]
    NotAllowed(String),
}

impl DomainError {
    #[must_use]
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    #[must_use]
    pub fn not_found(entity: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity: entity.into(),
            id: id.into(),
        }
    }

    #[must_use]
    pub fn duplicate(
        entity: impl Into<String>,
        field: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::Duplicate {
            entity: entity.into(),
            field: field.into(),
            value: value.into(),
        }
    }

    #[must_use]
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }

    #[must_use]
    pub fn not_allowed(msg: impl Into<String>) -> Self {
        Self::NotAllowed(msg.into())
    }
}