// crates/domain/users/src/lib.rs
//! User management for white label owners

use infrastructure_db::Database;
use infrastructure_session::{PasswordHasher, Session, SessionError, SessionStore};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found")]
    NotFound,

    #[error("Email already exists")]
    EmailExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Session error: {0}")]
    SessionError(#[from] SessionError),
}

// ============================================================================
// USER STRUCT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub white_label_id: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Editor,
    Viewer,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Viewer => "viewer",
        }
    }
}

// ============================================================================
// USER SERVICE
// ============================================================================

pub struct UserService {
    db: Database,
    session_store: SessionStore,
}

impl UserService {
    pub fn new(db: Database, session_store: SessionStore) -> Self {
        Self { db, session_store }
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        white_label_id: &str,
        email: &str,
        password: &str,
        role: UserRole,
    ) -> Result<String, UserError> {
        // Check if email already exists for this white label
        let existing = self.get_user_by_email(white_label_id, email).await;
        if existing.is_ok() {
            return Err(UserError::EmailExists);
        }

        // Hash password
        let password_hash = PasswordHasher::hash(password)
            .map_err(|e| UserError::SessionError(e))?;

        // Create user
        let query = r#"
            CREATE users CONTENT {
                white_label_id: $white_label_id,
                email: $email,
                password_hash: $password_hash,
                role: $role,
                is_active: true,
                created_at: time::now(),
                updated_at: time::now(),
                last_login_at: NONE
            }
        "#;

        let mut result = self
            .db
            .client()
            .query(query)
            .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
            .bind(("email", email))
            .bind(("password_hash", password_hash))
            .bind(("role", role.as_str()))
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let user: Option<User> = result
            .take(0)
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(user.ok_or(UserError::DatabaseError("Failed to create user".to_string()))?.id)
    }

    /// Get user by email (within white label)
    pub async fn get_user_by_email(
        &self,
        white_label_id: &str,
        email: &str,
    ) -> Result<User, UserError> {
        let query = r#"
            SELECT * FROM users 
            WHERE white_label_id = $white_label_id 
            AND email = $email 
            LIMIT 1
        "#;

        let mut result = self
            .db
            .client()
            .query(query)
            .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
            .bind(("email", email))
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let users: Vec<User> = result
            .take(0)
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        users.into_iter().next().ok_or(UserError::NotFound)
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<User, UserError> {
        let user: Option<User> = self
            .db
            .client()
            .select(("users", user_id))
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        user.ok_or(UserError::NotFound)
    }

    /// Get all users for a white label
    pub async fn get_users_for_white_label(
        &self,
        white_label_id: &str,
    ) -> Result<Vec<User>, UserError> {
        let query = r#"
            SELECT * FROM users 
            WHERE white_label_id = $white_label_id 
            ORDER BY created_at DESC
        "#;

        let mut result = self
            .db
            .client()
            .query(query)
            .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let users: Vec<User> = result
            .take(0)
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(users)
    }

    /// Authenticate user and create session
    pub async fn login(
        &self,
        white_label_id: &str,
        email: &str,
        password: &str,
    ) -> Result<String, UserError> {
        // Get user
        let user = self.get_user_by_email(white_label_id, email).await?;

        // Check if active
        if !user.is_active {
            return Err(UserError::InvalidCredentials);
        }

        // Verify password
        let valid = PasswordHasher::verify(password, &user.password_hash)
            .map_err(|e| UserError::SessionError(e))?;

        if !valid {
            return Err(UserError::InvalidCredentials);
        }

        // Update last login
        let update_query = "UPDATE $user_id SET last_login_at = time::now()";
        self.db
            .client()
            .query(update_query)
            .bind(("user_id", format!("users:{}", user.id)))
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Create session (7 days)
        let session = Session::new(user.id, user.email, false, 604800);
        let session_id = self.session_store.create(session).await?;

        Ok(session_id)
    }

    /// Update user
    pub async fn update_user(
        &self,
        user_id: &str,
        email: Option<&str>,
        role: Option<UserRole>,
        is_active: Option<bool>,
    ) -> Result<(), UserError> {
        let mut query = "UPDATE $user_id SET updated_at = time::now()".to_string();

        if email.is_some() {
            query.push_str(", email = $email");
        }
        if role.is_some() {
            query.push_str(", role = $role");
        }
        if is_active.is_some() {
            query.push_str(", is_active = $is_active");
        }

        let mut q = self
            .db
            .client()
            .query(query)
            .bind(("user_id", format!("users:{}", user_id)));

        if let Some(e) = email {
            q = q.bind(("email", e));
        }
        if let Some(r) = role {
            q = q.bind(("role", r.as_str()));
        }
        if let Some(a) = is_active {
            q = q.bind(("is_active", a));
        }

        q.await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Change password
    pub async fn change_password(
        &self,
        user_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), UserError> {
        // Get user
        let user = self.get_user(user_id).await?;

        // Verify old password
        let valid = PasswordHasher::verify(old_password, &user.password_hash)
            .map_err(|e| UserError::SessionError(e))?;

        if !valid {
            return Err(UserError::InvalidCredentials);
        }

        // Hash new password
        let new_hash = PasswordHasher::hash(new_password)
            .map_err(|e| UserError::SessionError(e))?;

        // Update
        let query = "UPDATE $user_id SET password_hash = $password_hash, updated_at = time::now()";

        self.db
            .client()
            .query(query)
            .bind(("user_id", format!("users:{}", user_id)))
            .bind(("password_hash", new_hash))
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Delete user
    pub async fn delete_user(&self, user_id: &str) -> Result<(), UserError> {
        self.db
            .client()
            .delete(("users", user_id))
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

// ============================================================================
// USER REGISTRATION/ONBOARDING
// ============================================================================

use infrastructure_email::EmailService;

pub struct UserOnboarding {
    user_service: UserService,
    email_service: EmailService,
}

impl UserOnboarding {
    pub fn new(user_service: UserService, email_service: EmailService) -> Self {
        Self {
            user_service,
            email_service,
        }
    }

    /// Register new white label owner (creates user + sends welcome email)
    pub async fn register_owner(
        &self,
        white_label_id: &str,
        domain: &str,
        email: &str,
        password: &str,
    ) -> Result<String, UserError> {
        // Create admin user
        let user_id = self
            .user_service
            .create_user(white_label_id, email, password, UserRole::Admin)
            .await?;

        // Send welcome email
        if let Err(e) = self.email_service.send_welcome(email, "Owner", domain, password) {
            tracing::error!("Failed to send welcome email: {}", e);
            // Continue anyway - user is created
        }

        Ok(user_id)
    }
}

// ============================================================================
// ACTIX-WEB ROUTES
// ============================================================================

use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use askama::Template;

#[derive(Template)]
#[template(path = "login.html")]
struct TenantLoginTemplate {
    domain: String,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct TenantLoginForm {
    email: String,
    password: String,
}

/// Tenant login page
pub async fn tenant_login_page(req: HttpRequest) -> ActixResult<HttpResponse> {
    // Extract domain from Host header
    let domain = req
        .headers()
        .get("Host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost")
        .to_string();

    let template = TenantLoginTemplate {
        domain,
        error: None,
    };

    let html = template
        .render()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

/// Tenant login submit
pub async fn tenant_login_submit(
    req: HttpRequest,
    form: web::Form<TenantLoginForm>,
    user_service: web::Data<UserService>,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    // Extract domain from Host header
    let domain = req
        .headers()
        .get("Host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost");

    // Get white label ID from domain
    let label = db
        .get_label_by_domain(domain)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("White label not found"))?;

    // Attempt login
    match user_service.login(&label.id, &form.email, &form.password).await {
        Ok(session_id) => {
            // Set session cookie
            Ok(HttpResponse::Found()
                .cookie(
                    actix_web::cookie::Cookie::build("tenant_session", session_id)
                        .path("/")
                        .http_only(true)
                        .max_age(actix_web::cookie::time::Duration::days(7))
                        .finish(),
                )
                .insert_header(("Location", "/dashboard"))
                .finish())
        }
        Err(_) => {
            let template = TenantLoginTemplate {
                domain: domain.to_string(),
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

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::get().to(tenant_login_page))
        .route("/login", web::post().to(tenant_login_submit));
}