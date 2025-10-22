use actix_web::{HttpResponse, Result};

pub async fn manage() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Tenant Admin Manage"))
}