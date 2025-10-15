#![forbid(unsafe_code)]
#![deny(warnings)]

use crate_core_observability::tracing_init;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_init::init();
    crate_application_web::server::run().await
}
