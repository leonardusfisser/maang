use actix_web::web;

/// Configure webmaster routes (framework administration)
pub fn configure(cfg: &mut web::ServiceConfig) {
    let _ = cfg.service(
        web::scope("/webmaster")
            .route("/dashboard", web::get().to(crate::web_handlers_webmaster::dashboard))
            .route("/tenants", web::get().to(crate::web_handlers_webmaster::tenants))
            .route("/metrics", web::get().to(crate::web_handlers_webmaster::metrics))
    );
}