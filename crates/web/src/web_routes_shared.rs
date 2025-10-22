use actix_web::{web, HttpResponse};

async fn test_handler() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("TEST ROUTE WORKS"))
}

async fn test_dashboard_direct(state: web::Data<crate::AppState>) -> actix_web::Result<HttpResponse> {
    crate::web_handlers_shared_admin::dashboard(state).await
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    tracing::info!("Configuring shared routes");

    // Public routes - NO SCOPE
    let _ = cfg.route("/", web::get().to(crate::web_handlers_shared_public::home));
    let _ = cfg.route("/about", web::get().to(crate::web_handlers_shared_public::about));
    let _ = cfg.route("/contact", web::get().to(crate::web_handlers_shared_public::contact));
    let _ = cfg.route("/products", web::get().to(crate::web_handlers_shared_public::products));
    let _ = cfg.route("/services", web::get().to(crate::web_handlers_shared_public::services));
    let _ = cfg.route("/media", web::get().to(crate::web_handlers_shared_public::media));
    let _ = cfg.route("/login", web::get().to(crate::web_handlers_shared_public::login_page));
    let _ = cfg.route("/login", web::post().to(crate::web_handlers_shared_public::login_post));
    let _ = cfg.route("/signup", web::get().to(crate::web_handlers_shared_public::signup_page));
    let _ = cfg.route("/signup", web::post().to(crate::web_handlers_shared_public::signup_post));

    tracing::info!("Configuring admin routes");

    // Test routes
    let _ = cfg.route("/admintest", web::get().to(test_handler));
    let _ = cfg.route("/admindash", web::get().to(test_dashboard_direct));

    // Admin routes with proper scope
    let _ = cfg.service(
        web::scope("/admin")
            .route("/dashboard", web::get().to(crate::web_handlers_shared_admin::dashboard))
            .route("/users", web::get().to(crate::web_handlers_shared_admin::users))
            .route("/config", web::get().to(crate::web_handlers_shared_admin::config))
            .route("/settings", web::get().to(crate::web_handlers_shared_admin::settings))
            .route("/metrics", web::get().to(crate::web_handlers_shared_admin::metrics))
            .route("/version", web::get().to(crate::web_handlers_shared_admin::version))
    );

    tracing::info!("Routes configured");
}