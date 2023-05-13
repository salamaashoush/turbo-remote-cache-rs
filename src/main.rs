pub mod api;
mod helpers;
pub mod storage;

use actix_cors::Cors;
use actix_web::{
    http::KeepAlive,
    middleware::Logger,
    web::{scope, PayloadConfig},
    App, HttpServer,
};

use api::artifacts;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .service(scope("/v8").configure(artifacts::config))
            .app_data(PayloadConfig::new(104857600))
    })
    .keep_alive(KeepAlive::Os)
    .bind(("127.0.0.1", 4000))?
    .run()
    .await
}
