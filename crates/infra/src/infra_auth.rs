//! Authentication business logic
//!
//! Handles login/signup flows, session creation, and authentication state

use domain::Session;
use uuid::Uuid;

/// Generate a unique session ID
pub fn generate_session_id() -> String {
    Uuid::new_v4().to_string()
}

/// Create and store a new session for authenticated user
pub async fn create_user_session(
    redis: &mut redis::aio::ConnectionManager,
    user_id: String,
    tenant_id: String,
    session_ttl_secs: u64,
    csrf_token_ttl_secs: u64,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Generate IDs
    let session_id = generate_session_id();
    let csrf_token = infra::generate_token()?;

    // Create session
    let session = Session::new(
        session_id.clone(),
        user_id,
        tenant_id,
        csrf_token.clone(),
        session_ttl_secs,
    );

    // Store in Redis
    infra::store_session(redis, &session).await?;
    infra::store_token(redis, &session_id, &csrf_token, csrf_token_ttl_secs).await?;

    Ok((session_id, csrf_token))
}

/// Build secure session cookie header
pub fn build_session_cookie(session_id: &str) -> String {
    format!(
        "session_id={}; Path=/; HttpOnly; SameSite=Strict; Secure",
        session_id
    )
}