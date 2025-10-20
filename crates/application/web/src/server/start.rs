// crates/application/web/src/server/start.rs

use actix_web::{App, HttpServer, web, HttpResponse};
use actix_files::Files;
use crate::router::{routes_labeled, routes_shared, routes_webmaster};
use tracing::{error, info};

// Embed default.css at compile time
const DEFAULT_CSS: &str = include_str!("../../static/css/default.css");

/// Handler to serve the embedded default.css
async fn serve_default_css() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(DEFAULT_CSS)
}

/// Starts the Actix Web server.
///
/// # Errors
///
/// Returns an error if:
/// - The server fails to bind to the specified address
/// - A runtime error occurs during server execution
pub async fn run(bind_addr: &str) -> std::io::Result<()> {
    info!(target: "server", "Starting Actix Web server at {}", bind_addr);

    let server = HttpServer::new(move || {
        App::new()
            // Serve embedded default.css
            .service(web::resource("/static/css/default.css").route(web::get().to(serve_default_css)))
            // Serve static files from the app's static directory
            .service(Files::new("/static", "./static").show_files_listing())
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
