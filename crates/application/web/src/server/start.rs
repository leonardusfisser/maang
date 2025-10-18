// crates/application/web/src/server/start.rs

use actix_web::{App, HttpServer, web};
use crate::router::{ routes_labeled, routes_shared,routes_webmaster };
use tracing::{info, error};


pub async fn run(bind_addr: &str) -> std::io::Result<()> {
    info!(target: "server", "Starting Actix Web server at {}", bind_addr);

    let server = HttpServer::new(move || {
        App::new()
            .configure(routes_shared::routing_shared)
            .configure(routes_labeled::routing_labeled)
            .configure(routes_webmaster::routing_webmaster)
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
