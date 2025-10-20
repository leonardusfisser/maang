use actix_web::{HttpResponse, Responder};

/// GET /login - Show login page
pub async fn get() -> impl Responder {
    HttpResponse::Ok().body("Login page - TODO: implement with Askama template")
}