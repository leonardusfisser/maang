use actix_web::{HttpResponse, Result};

pub async fn dashboard() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Webmaster Dashboard"))
}

pub async fn tenants() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Webmaster Tenants"))
}

pub async fn metrics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Webmaster Metrics"))
}