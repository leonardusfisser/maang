use actix_web::{HttpResponse, Responder};
use std::sync::atomic::{AtomicU64, Ordering};

static REQ_COUNT: AtomicU64 = AtomicU64::new(0);

/// Simple GET /metrics exporter (Prometheus-like).
pub async fn get() -> impl Responder {
    let count = REQ_COUNT.fetch_add(1, Ordering::Relaxed);
    let body = format!(
        "# HELP http_requests_total Total requests handled\n\
         # TYPE http_requests_total counter\n\
         http_requests_total {count}\n"
    );
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(body)
}
