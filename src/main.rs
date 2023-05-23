pub mod api;
pub mod auth;
pub mod config;
pub mod helpers;
pub mod storage;

use actix_cors::Cors;
use actix_web::{
    middleware::Logger,
    web::{scope, Data, PayloadConfig},
    App, HttpServer,
};
use std::{
    env::{args, set_var},
    path::Path,
    sync::Mutex,
};

use api::artifacts;
use storage::StorageStore;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env_file = args().nth(1).unwrap_or(".env".to_string());
    dotenv::from_path(Path::new(&env_file)).ok();
    set_var("RUST_LOG", "info");
    set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let storage = Data::new(Mutex::new(StorageStore::new()));
    HttpServer::new(move || {
        App::new()
            .wrap(auth::Auth)
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .app_data(storage.clone())
            .service(scope("/v8").configure(artifacts::config))
            .app_data(PayloadConfig::new(104857600))
    })
    .bind(("0.0.0.0", config::get_port()))?
    .run()
    .await
}
