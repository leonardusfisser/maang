use actix_web::{HttpResponse, Responder};
use serde::Serialize;

/// Version information returned by /version
#[derive(Serialize)]
struct VersionInfo {
    version: &'static str,
    git: &'static str,
    build_time: &'static str,
}

/// GET /version
pub async fn get() -> impl Responder {
    HttpResponse::Ok().json(VersionInfo {
        version: env!("CARGO_PKG_VERSION"),
        git: option_env!("GIT_HASH").unwrap_or("unknown"),
        build_time: option_env!("BUILD_TIME").unwrap_or("unknown"),
    })
}
