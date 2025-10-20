#[actix_web::main]
async fn main() -> std::io::Result<()> {
    core_observability::tracing_init::init();

    let _ = core_config::load_env("framerust");

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8082".to_string());
    let addr = format!("{host}:{port}");

    application_web::server::run(&addr).await
}
