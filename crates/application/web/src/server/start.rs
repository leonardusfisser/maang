// crates/application/web/src/server/start.rs

use actix_web::{App, HttpServer, web};
use core_observability::tracing_init;
use tracing::{info, error};

use crate::routes; // assuming you have web::routes::configure()

pub async fn run(bind_addr: &str) -> std::io::Result<()> {
    // ensure tracing already set by main, but safe to call again
    tracing_init::init();

    info!(target: "server", "Starting Actix Web server at {}", bind_addr);

    let server = HttpServer::new(move || {
        App::new()
            .configure(routes::configure)
    })
        .bind(bind_addr)
        .map_err(|e| {
            error!(error = ?e, "failed to bind server");
            e
        })?;

    server
        .run()
        .await
        .map_err(|e| {
            error!(error = ?e, "server runtime error");
            e
        })
}
