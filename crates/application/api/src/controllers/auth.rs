use actix_web::{post, web, HttpRequest, HttpResponse};
use domain_sessions::Session;
use infrastructure_session::RedisSessionStore;
use infrastructure_ratelimit::RateLimiter;
use infrastructure_csrf::CsrfToken;
use core_errors::{SessionError, SecurityError};
use tracing::{instrument, info, warn};
use rand::{Rng, distr::Alphanumeric};

#[post("/login")]
#[instrument(skip(store, limiter, req))]
async fn login(
    store: web::Data<RedisSessionStore>,
    limiter: web::Data<RateLimiter>,
    csrf_secret: web::Data<String>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let ip = req.connection_info().realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    limiter.check_login(&ip).await
        .map_err(|_| actix_web::error::ErrorTooManyRequests("Rate limit exceeded"))?;

    // Verify credentials here (check SurrealDB)
    // If invalid, return early (don't reset rate limit)

    let token: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let session = Session::new("user_123".into(), "tenant_456".into());
    store.create(&token, &session, 1800).await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Session creation failed"))?;

    limiter.reset_login(&ip).await.ok();

    let csrf_token = CsrfToken::generate(&token, &csrf_secret);

    info!("User logged in: {}", session.user_id);

    Ok(HttpResponse::Ok()
        .cookie(
            actix_web::cookie::Cookie::build("session", token)
                .path("/")
                .secure(true)
                .http_only(true)
                .same_site(actix_web::cookie::SameSite::Strict)
                .max_age(actix_web::cookie::time::Duration::seconds(1800))
                .finish()
        )
        .json(serde_json::json!({ "csrf_token": csrf_token })))
}

#[post("/logout")]
#[instrument(skip(store, req))]
async fn logout(
    store: web::Data<RedisSessionStore>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    if let Some(cookie) = req.cookie("session") {
        store.delete(cookie.value()).await.ok();
        info!("User logged out");
    }

    Ok(HttpResponse::Ok()
        .cookie(
            actix_web::cookie::Cookie::build("session", "")
                .path("/")
                .max_age(actix_web::cookie::time::Duration::ZERO)
                .finish()
        )
        .finish())
}

#[post("/rotate-session")]
#[instrument(skip(store, csrf_secret, req))]
async fn rotate_session(
    store: web::Data<RedisSessionStore>,
    csrf_secret: web::Data<String>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let old_token = req.cookie("session")
        .map(|c| c.value().to_string())
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("No session"))?;

    let new_token = store.rotate(&old_token, 1800).await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Rotation failed"))?;

    let csrf_token = CsrfToken::generate(&new_token, &csrf_secret);

    info!("Session rotated");

    Ok(HttpResponse::Ok()
        .cookie(
            actix_web::cookie::Cookie::build("session", new_token)
                .path("/")
                .secure(true)
                .http_only(true)
                .same_site(actix_web::cookie::SameSite::Strict)
                .max_age(actix_web::cookie::time::Duration::seconds(1800))
                .finish()
        )
        .json(serde_json::json!({ "csrf_token": csrf_token })))
}