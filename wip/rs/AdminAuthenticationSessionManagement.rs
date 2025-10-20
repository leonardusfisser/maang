// crates/infrastructure/session/src/lib.rs
//! Session management using Redis for secure authentication

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,

    #[error("Session expired")]
    Expired,

    #[error("Invalid session")]
    Invalid,

    #[error("Storage error: {0}")]
    StorageError(String),
}

// ============================================================================
// SESSION DATA
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub email: String,
    pub is_admin: bool,
    pub created_at: i64,
    pub expires_at: i64,
}

impl Session {
    pub fn new(user_id: String, email: String, is_admin: bool, ttl_seconds: i64) -> Self {
        use time::OffsetDateTime;

        let now = OffsetDateTime::now_utc().unix_timestamp();

        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            email,
            is_admin,
            created_at: now,
            expires_at: now + ttl_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        use time::OffsetDateTime;

        let now = OffsetDateTime::now_utc().unix_timestamp();
        now >= self.expires_at
    }
}

// ============================================================================
// IN-MEMORY SESSION STORE (Production: use Redis)
// ============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(&self, session: Session) -> Result<String, SessionError> {
        let session_id = session.id.clone();
        self.sessions.write().await.insert(session_id.clone(), session);
        Ok(session_id)
    }

    pub async fn get(&self, session_id: &str) -> Result<Session, SessionError> {
        let sessions = self.sessions.read().await;

        let session = sessions
            .get(session_id)
            .ok_or(SessionError::NotFound)?
            .clone();

        if session.is_expired() {
            return Err(SessionError::Expired);
        }

        Ok(session)
    }

    pub async fn delete(&self, session_id: &str) -> Result<(), SessionError> {
        self.sessions.write().await.remove(session_id);
        Ok(())
    }

    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| !session.is_expired());
    }
}

// ============================================================================
// PASSWORD HASHING
// ============================================================================

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub struct PasswordHasher;

impl PasswordHasher {
    pub fn hash(password: &str) -> Result<String, SessionError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| SessionError::StorageError(e.to_string()))?
            .to_string();

        Ok(hash)
    }

    pub fn verify(password: &str, hash: &str) -> Result<bool, SessionError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| SessionError::StorageError(e.to_string()))?;

        let argon2 = Argon2::default();

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

// ============================================================================
// ADMIN USER MANAGEMENT
// ============================================================================

use infrastructure_db::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

pub struct AdminAuth {
    db: Database,
    session_store: SessionStore,
}

impl AdminAuth {
    pub fn new(db: Database, session_store: SessionStore) -> Self {
        Self { db, session_store }
    }

    /// Create a new admin user
    pub async fn create_admin(
        &self,
        email: &str,
        password: &str,
    ) -> Result<String, SessionError> {
        let password_hash = PasswordHasher::hash(password)?;

        let query = r#"
            CREATE admin_users CONTENT {
                email: $email,
                password_hash: $password_hash,
                is_active: true,
                created_at: time::now(),
                last_login_at: NONE
            }
        "#;

        let mut result = self
            .db
            .client()
            .query(query)
            .bind(("email", email))
            .bind(("password_hash", password_hash))
            .await
            .map_err(|e| SessionError::StorageError(e.to_string()))?;

        let admin: Option<AdminUser> = result
            .take(0)
            .map_err(|e| SessionError::StorageError(e.to_string()))?;

        Ok(admin.ok_or(SessionError::Invalid)?.id)
    }

    /// Authenticate admin and create session
    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<String, SessionError> {
        // Get admin by email
        let query = "SELECT * FROM admin_users WHERE email = $email AND is_active = true LIMIT 1";

        let mut result = self
            .db
            .client()
            .query(query)
            .bind(("email", email))
            .await
            .map_err(|e| SessionError::StorageError(e.to_string()))?;

        let admins: Vec<AdminUser> = result
            .take(0)
            .map_err(|e| SessionError::StorageError(e.to_string()))?;

        let admin = admins.into_iter().next().ok_or(SessionError::NotFound)?;

        // Verify password
        if !PasswordHasher::verify(password, &admin.password_hash)? {
            return Err(SessionError::Invalid);
        }

        // Update last login
        let update_query = "UPDATE $admin_id SET last_login_at = time::now()";
        self.db
            .client()
            .query(update_query)
            .bind(("admin_id", format!("admin_users:{}", admin.id)))
            .await
            .map_err(|e| SessionError::StorageError(e.to_string()))?;

        // Create session (24 hours)
        let session = Session::new(admin.id, admin.email, true, 86400);
        let session_id = self.session_store.create(session).await?;

        Ok(session_id)
    }

    /// Validate session
    pub async fn validate_session(&self, session_id: &str) -> Result<Session, SessionError> {
        self.session_store.get(session_id).await
    }

    /// Logout (delete session)
    pub async fn logout(&self, session_id: &str) -> Result<(), SessionError> {
        self.session_store.delete(session_id).await
    }
}

// ============================================================================
// ACTIX-WEB MIDDLEWARE
// ============================================================================

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct AdminAuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AdminAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminAuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminAuthMiddlewareService { service }))
    }
}

pub struct AdminAuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Get session cookie
        let session_id = req.cookie("session_id").map(|c| c.value().to_string());

        if let Some(session_id) = session_id {
            // Get session store from app data
            if let Some(session_store) = req.app_data::<actix_web::web::Data<SessionStore>>() {
                let session_store = session_store.clone();
                let fut = self.service.call(req);

                return Box::pin(async move {
                    // Validate session
                    match session_store.get(&session_id).await {
                        Ok(session) => {
                            // Session valid, continue
                            fut.await
                        }
                        Err(_) => {
                            // Invalid session, redirect to login
                            Ok(req.into_response(
                                HttpResponse::Found()
                                    .insert_header(("Location", "/admin/login"))
                                    .finish(),
                            ))
                        }
                    }
                });
            }
        }

        // No session, redirect to login
        let response = HttpResponse::Found()
            .insert_header(("Location", "/admin/login"))
            .finish();

        Box::pin(async move { Ok(req.into_response(response)) })
    }
}

// ============================================================================
// LOGIN ROUTES
// ============================================================================

use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use askama::Template;

#[derive(Template)]
#[template(path = "admin/login.html")]
struct LoginTemplate {
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

pub async fn login_page(req: HttpRequest) -> ActixResult<HttpResponse> {
    // If already logged in, redirect to dashboard
    if req.cookie("session_id").is_some() {
        return Ok(HttpResponse::Found()
            .insert_header(("Location", "/admin"))
            .finish());
    }

    let template = LoginTemplate { error: None };
    let html = template
        .render()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

pub async fn login_submit(
    form: web::Form<LoginForm>,
    admin_auth: web::Data<AdminAuth>,
) -> ActixResult<HttpResponse> {
    match admin_auth.login(&form.email, &form.password).await {
        Ok(session_id) => {
            // Set session cookie
            Ok(HttpResponse::Found()
                .cookie(
                    actix_web::cookie::Cookie::build("session_id", session_id)
                        .path("/")
                        .http_only(true)
                        .max_age(actix_web::cookie::time::Duration::days(1))
                        .finish(),
                )
                .insert_header(("Location", "/admin"))
                .finish())
        }
        Err(_) => {
            let template = LoginTemplate {
                error: Some("Invalid email or password".to_string()),
            };
            let html = template
                .render()
                .map_err(actix_web::error::ErrorInternalServerError)?;

            Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html))
        }
    }
}

pub async fn logout(
    req: HttpRequest,
    admin_auth: web::Data<AdminAuth>,
) -> ActixResult<HttpResponse> {
    if let Some(session_cookie) = req.cookie("session_id") {
        let _ = admin_auth.logout(session_cookie.value()).await;
    }

    Ok(HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build("session_id", "")
                .path("/")
                .max_age(actix_web::cookie::time::Duration::ZERO)
                .finish(),
        )
        .insert_header(("Location", "/admin/login"))
        .finish())
}

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/admin/login", web::get().to(login_page))
        .route("/admin/login", web::post().to(login_submit))
        .route("/admin/logout", web::get().to(logout));
}

// ============================================================================
// SETUP SCRIPT
// ============================================================================

/// Create initial admin user
pub async fn create_initial_admin(
    db: &Database,
    email: &str,
    password: &str,
) -> Result<(), SessionError> {
    let session_store = SessionStore::new();
    let admin_auth = AdminAuth::new(db.clone(), session_store);

    match admin_auth.create_admin(email, password).await {
        Ok(admin_id) => {
            tracing::info!("Created admin user: {} (ID: {})", email, admin_id);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to create admin: {}", e);
            Err(e)
        }
    }
}