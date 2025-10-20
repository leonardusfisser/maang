use actix_web::{HttpResponse, Responder, web};
use crate::handlers_shared::{self, public_base};

/// Register all public + admin routes for `LillPepe`.
pub fn routing_shared(cfg: &mut web::ServiceConfig) {

    // PUBLIC
    cfg.service(web::resource("/").route(web::get().to(public_base::home)));
    cfg.service(web::resource("/about").route(web::get().to(handlers_shared::public_about::get)));
    cfg.service(web::resource("/products").route(web::get().to(handlers_shared::public_products::list)));
    cfg.service(web::resource("/services").route(web::get().to(handlers_shared::public_services::list)));
    cfg.service(web::resource("/media").route(web::get().to(handlers_shared::public_media::list)));
    cfg.service(web::resource("/contact").route(web::get().to(handlers_shared::public_contact::get)));


    // AUTH
    cfg.service(web::resource("/login").route(web::get().to(handlers_shared::public_login::get)));
    cfg.service(web::resource("/register").route(web::get().to(handlers_shared::public_signup::get)));


    // ADMIN
    cfg.service(web::resource("/dashboard").route(web::get().to(handlers_shared::admin_dashboard::get)));
    cfg.service(web::resource("/dashboard").route(web::get().to(handlers_shared::admin_config::get)));
    cfg.service(web::resource("/settings").route(web::get().to(handlers_shared::admin_settings::get)));
    cfg.service(web::resource("/users").route(web::get().to(handlers_shared::admin_users::list)));
    cfg.service(web::resource("/logout").route(web::post().to(handlers_shared::admin_logout::post)));

    cfg.service(web::resource("/health").route(web::get().to(health)));
    cfg.service(web::resource("/metrics").route(web::get().to(handlers_shared::admin_metrics::get)));
    cfg.service(web::resource("/version").route(web::get().to(handlers_shared::admin_version::get)));
}

async fn health() -> impl Responder {
    HttpResponse::Ok().body("ok")
}
