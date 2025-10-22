use actix_web::web;

/// Configure tenant-specific routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    let _ = cfg.service(
        web::scope("/tenants")
            .route("/shop", web::get().to(crate::web_handlers_tenants_public::shop))
            .route("/auctions", web::get().to(crate::web_handlers_tenants_public::auctions))
            .route("/properties", web::get().to(crate::web_handlers_tenants_public::properties))
    );
    let _ = cfg.service(
        web::scope("/tenants/admin")
            .route("/manage", web::get().to(crate::web_handlers_tenants_admin::manage))
    );
}