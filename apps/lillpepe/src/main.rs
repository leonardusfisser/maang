//! LillPepe main binary – starts the web server.

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    core_observability::tracing_init::init();
    application_web::server::run("127.0.0.1:8080").await
}
