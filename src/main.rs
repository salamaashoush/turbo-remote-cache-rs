pub mod api;
pub mod auth;
pub mod config;
pub mod helpers;
pub mod storage;

use actix_cors::Cors;
use actix_web::{
  middleware::Logger,
  web::{get, scope, Data, PayloadConfig},
  App, HttpServer,
};
use std::{
  env::{args, set_var},
  path::Path,
};

use api::{artifacts, turborepo};

use storage::StorageStore;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let env_file = args().nth(1).unwrap_or(".env".to_string());
  dotenvy::from_path(Path::new(&env_file)).ok();
  set_var("RUST_LOG", "info");
  set_var("RUST_BACKTRACE", "1");
  env_logger::init();

  HttpServer::new(move || {
    App::new()
      .wrap(Logger::default())
      .wrap(
        Cors::default()
          .allow_any_header()
          .allow_any_method()
          .allow_any_origin(),
      )
      .app_data(Data::new(StorageStore::new()))
      .configure(turborepo::config)
      .service(
        scope("/v8")
          .route("/artifacts/status", get().to(artifacts::get_status))
          .configure(artifacts::config),
      )
      .app_data(PayloadConfig::new(104857600))
  })
  .bind(("0.0.0.0", config::get_port()))?
  .run()
  .await
}
