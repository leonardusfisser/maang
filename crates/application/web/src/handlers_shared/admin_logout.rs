use actix_web::{HttpResponse, Responder};

pub async fn post() -> impl Responder {
    HttpResponse::Ok().body("logged out")
}
