#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Entity already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    #[error("Tenant error: {0}")]
    Tenant(String),

    #[error("User error: {0}")]
    User(String),

    #[error("Product error: {0}")]
    Product(String),

    #[error("Service error: {0}")]
    Service(String),

    #[error("Media error: {0}")]
    Media(String),
}

impl DomainError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn not_found(entity: impl Into<String>) -> Self {
        Self::NotFound(entity.into())
    }

    pub fn already_exists(entity: impl Into<String>) -> Self {
        Self::AlreadyExists(entity.into())
    }

    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }

    pub fn business_rule(msg: impl Into<String>) -> Self {
        Self::BusinessRule(msg.into())
    }
}