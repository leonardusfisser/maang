use actix_web::{HttpResponse, Responder};

pub async fn list() -> impl Responder {
    HttpResponse::Ok().body("services")
}
