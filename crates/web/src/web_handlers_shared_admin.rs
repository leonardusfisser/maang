use actix_web::{web, HttpResponse, Result};
use askama::Template;
use crate::web_types::{DashboardTemplate, UsersTemplate};

pub async fn dashboard(state: web::Data<crate::AppState>) -> Result<HttpResponse> {
    tracing::info!("Dashboard handler called");

    let tenant_id = "00000000-0000-0000-0000-000000000001";

    tracing::info!("Fetching products for tenant: {}", tenant_id);
    let products = infra::list_products(&state.db, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch products: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    tracing::info!("Fetching services for tenant: {}", tenant_id);
    let services = infra::list_services(&state.db, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch services: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    tracing::info!("Rendering dashboard with {} products, {} services", products.len(), services.len());

    let template = DashboardTemplate {
        tenant_name: "MaangFrame".to_string(),
        user_email: "admin@maangframe.com".to_string(),
        product_count: products.len(),
        service_count: services.len(),
    };

    let html = template.render()
        .map_err(|e| {
            tracing::error!("Failed to render template: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    tracing::info!("Dashboard rendered successfully");
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn users(_state: web::Data<crate::AppState>) -> Result<HttpResponse> {
    let template = UsersTemplate {
        tenant_name: "MaangFrame".to_string(),
        user_email: "admin@maangframe.com".to_string(),
    };

    let html = template.render()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub async fn config() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Admin Config"))
}

pub async fn settings() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Admin Settings"))
}

pub async fn metrics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Admin Metrics"))
}

pub async fn version() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Version: 0.1.0"))
}