use actix_web::{web, HttpResponse, Result};
use askama::Template;
use crate::web_types::{
    HomeTemplate, AboutTemplate, ContactTemplate, ProductsTemplate,
    ServicesTemplate, MediaTemplate, LoginTemplate, SignupTemplate,
    LoginForm, SignupForm,
};

// ============================================================================
// PUBLIC PAGES (No Auth Required)
// ============================================================================

pub async fn home() -> Result<HttpResponse> {
    let template = HomeTemplate {
        tenant_name: "MaangFrame".to_string(),
        tagline: "Production-grade SaaS Framework".to_string(),
        background_video: "/assets/backgrounds/background.mp4".to_string(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn about() -> Result<HttpResponse> {
    let template = AboutTemplate {
        tenant_name: "MaangFrame".to_string(),
        description: "Built with Rust for maximum performance and security".to_string(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn contact() -> Result<HttpResponse> {
    let template = ContactTemplate {
        tenant_name: "MaangFrame".to_string(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn products() -> Result<HttpResponse> {
    let template = ProductsTemplate {
        tenant_name: "MaangFrame".to_string(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn services() -> Result<HttpResponse> {
    let template = ServicesTemplate {
        tenant_name: "MaangFrame".to_string(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn media() -> Result<HttpResponse> {
    let template = MediaTemplate {
        tenant_name: "MaangFrame".to_string(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

// ============================================================================
// AUTHENTICATION - LOGIN
// ============================================================================

pub async fn login_page() -> Result<HttpResponse> {
    let csrf_token = infra::generate_token()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let template = LoginTemplate {
        tenant_name: "MaangFrame".to_string(),
        csrf_token,
        error: String::new(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn login_post(
    state: web::Data<crate::AppState>,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse> {
    tracing::info!("Login attempt for email: {}", form.email);

    // Step 1: Find user by email
    let user = match infra::find_by_email(&state.db, &form.email).await {
        Ok(Some(u)) => {
            tracing::info!("User found: {}", u.email);
            u
        }
        Ok(None) => {
            tracing::warn!("User not found: {}", form.email);
            return render_login_error("Invalid email or password");
        }
        Err(e) => {
            tracing::error!("Database error: {:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e));
        }
    };

    // Step 2: Verify password using infra_pwd
    if !infra_pwd::verify_password(&form.password, &user.salt, &user.password_hash) {
        tracing::warn!("Invalid password for: {}", user.email);
        return render_login_error("Invalid email or password");
    }

    // Step 3: Create session using infra_auth
    let mut redis = state.redis.clone();
    let (session_id, _csrf_token) = infra_auth::create_user_session(
        &mut redis,
        user.id,
        user.tenant_id,
        state.config.security.session_ttl_secs,
        state.config.security.csrf_token_ttl_secs,
    )
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Step 4: Build cookie and redirect
    let cookie = infra_auth::build_session_cookie(&session_id);

    tracing::info!("Login successful for: {}", user.email);
    Ok(HttpResponse::Found()
        .append_header(("Location", "/admin/dashboard"))
        .append_header(("Set-Cookie", cookie))
        .finish())
}

/// Helper to render login page with error message
fn render_login_error(error_msg: &str) -> Result<HttpResponse> {
    let csrf_token = infra::generate_token()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let template = LoginTemplate {
        tenant_name: "MaangFrame".to_string(),
        csrf_token,
        error: error_msg.to_string(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

// ============================================================================
// AUTHENTICATION - SIGNUP
// ============================================================================

pub async fn signup_page() -> Result<HttpResponse> {
    let csrf_token = infra::generate_token()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let template = SignupTemplate {
        tenant_name: "MaangFrame".to_string(),
        csrf_token,
        error: String::new(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn signup_post(
    state: web::Data<crate::AppState>,
    form: web::Form<SignupForm>,
) -> Result<HttpResponse> {
    // Step 1: Validate password match
    if form.password != form.password_confirm {
        return render_signup_error("Passwords do not match");
    }

    // Step 2: Check if email already exists
    let existing = infra::find_by_email(&state.db, &form.email)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    if existing.is_some() {
        return render_signup_error("Email already registered");
    }

    // Step 3: Hash password using infra_pwd
    let salt = infra_pwd::generate_salt();
    let password_hash = infra_pwd::hash_password(&form.password, &salt);

    // Step 4: Create user in database
    let create_user = domain::CreateUser {
        tenant_id: "00000000-0000-0000-0000-000000000001".to_string(),
        email: form.email.clone(),
        password: form.password.clone(),
        role: domain::UserRole::User,
    };

    let user = infra::create_user(&state.db, &create_user, &password_hash, &salt)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Step 5: Create session using infra_auth
    let mut redis = state.redis.clone();
    let (session_id, _csrf_token) = infra_auth::create_user_session(
        &mut redis,
        user.id,
        user.tenant_id,
        state.config.security.session_ttl_secs,
        state.config.security.csrf_token_ttl_secs,
    )
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Step 6: Build cookie and redirect
    let cookie = infra_auth::build_session_cookie(&session_id);

    tracing::info!("Signup successful for: {}", user.email);
    Ok(HttpResponse::Found()
        .append_header(("Location", "/admin/dashboard"))
        .append_header(("Set-Cookie", cookie))
        .finish())
}

/// Helper to render signup page with error message
fn render_signup_error(error_msg: &str) -> Result<HttpResponse> {
    let csrf_token = infra::generate_token()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let template = SignupTemplate {
        tenant_name: "MaangFrame".to_string(),
        csrf_token,
        error: error_msg.to_string(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}