use actix_web::{HttpResponse, Result};

pub async fn shop() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Tenant Shop"))
}

pub async fn auctions() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Tenant Auctions"))
}

pub async fn properties() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Tenant Properties"))
}