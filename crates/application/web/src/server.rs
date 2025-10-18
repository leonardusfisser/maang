use crate::configure;
use actix_web::{App, HttpServer};
// your existing configure()

pub async fn run(bind: &str) -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(configure))
        .bind(bind)?
        .run()
        .await
}
