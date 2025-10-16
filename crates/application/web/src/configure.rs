use actix_web::{web, HttpResponse, Responder};

/// Register all public + admin routes for LillPepe.
pub fn configure(cfg: &mut web::ServiceConfig) {
    // Public pages
    cfg.service(web::resource("/").route(web::get().to(handlers::base::home)));
    cfg.service(web::resource("/about").route(web::get().to(handlers::about::get)));
    cfg.service(web::resource("/products").route(web::get().to(handlers::products::list)));
    cfg.service(web::resource("/services").route(web::get().to(handlers::services::list)));
    cfg.service(web::resource("/contact").route(web::get().to(handlers::contact::get)));
    cfg.service(web::resource("/photos").route(web::get().to(handlers::photos::list)));
    cfg.service(web::resource("/videos").route(web::get().to(handlers::videos::list)));
    cfg.service(web::resource("/posts").route(web::get().to(handlers::posts::list)));

    // Auth
    cfg.service(web::resource("/login").route(web::get().to(handlers::login::get)));
    cfg.service(web::resource("/register").route(web::get().to(handlers::signup::get)));
    cfg.service(web::resource("/logout").route(web::post().to(handlers::logout::post)));

    // Admin / Dashboard
    cfg.service(web::resource("/admin").route(web::get().to(handlers::admin::get)));
    cfg.service(web::resource("/dashboard").route(web::get().to(handlers::dashboard::get)));
    cfg.service(web::resource("/settings").route(web::get().to(handlers::settings::get)));

    // Tenants / Users
    cfg.service(web::resource("/tenants").route(web::get().to(handlers::tenants::list)));
    cfg.service(web::resource("/users").route(web::get().to(handlers::users::list)));

    // Media + docs
    cfg.service(web::resource("/media").route(web::get().to(handlers::media::list)));
    cfg.service(web::resource("/pdf").route(web::get().to(handlers::pdf::list)));
    cfg.service(web::resource("/audio").route(web::get().to(handlers::audio::list)));

    // Health
    cfg.service(web::resource("/health").route(web::get().to(health)));
}

/// Lightweight built-in health handler (safe even if handlers::health isn’t ready)
async fn health() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

/// Short module path aliases so `configure()` reads cleanly.
mod handlers {
    pub use crate::handlers::{
        about, admin, audio, base, contact, dashboard, login, logout, media, pdf, photos, posts,
        products, services, settings, signup, tenants, users, videos,
    };
}
