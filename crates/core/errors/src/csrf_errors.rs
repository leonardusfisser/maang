#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("CSRF token invalid")]
    CsrfInvalid,
    #[error("Rate limit exceeded")]
    RateLimited,
    #[error("Tenant mismatch")]
    TenantMismatch,
}