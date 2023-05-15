pub mod api;
pub mod auth;
pub mod config;
pub mod helpers;
pub mod storage;

use actix_cors::Cors;
use actix_web::{
    middleware::{ErrorHandlers, Logger},
    web::{scope, Data, PayloadConfig},
    App, HttpServer,
};
use std::sync::Mutex;

use api::artifacts;
use storage::StorageStore;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    // let storage: Arc<StorageStore> = Arc::new(StorageStore::new());
    let storage = Data::new(Mutex::new(StorageStore::new()));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(ErrorHandlers::default())
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .wrap(auth::Auth)
            .app_data(storage.clone())
            .service(scope("/v8").configure(artifacts::config))
            .app_data(PayloadConfig::new(104857600))
    })
    .bind(("127.0.0.1", config::get_port()))?
    .run()
    .await
}
