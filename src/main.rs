use actix_cors::Cors;
use actix_web::{
  middleware::Logger,
  web::{Data, PayloadConfig},
  App, HttpServer,
};
use log::info;
use std::{env::args, path::Path, sync::Arc};

use crate::config::{get_port, Config};
use crate::handlers::{artifacts, turborepo};

pub mod auth;
pub mod config;
pub mod handlers;
pub mod helpers;
pub mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  // Load the environment variables from the .env file
  let env_file = args().nth(1).unwrap_or(".env".to_string());
  dotenvy::from_path(Path::new(&env_file)).ok();
  // Initialize the logger
  env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
  let config = Arc::new(Config::from_env().expect("error loading config from environment"));
  let port = get_port();
  info!("starting HTTP server at http://localhost:{}", port);
  // Create and Start the HTTP server
  HttpServer::new(move || {
    App::new()
      .wrap(Logger::default())
      .wrap(
        Cors::default()
          .allow_any_header()
          .allow_any_method()
          .allow_any_origin(),
      )
      .app_data(Data::new(config.clone()))
      .configure(turborepo::configure)
      .configure(artifacts::configure(&config))
      .app_data(PayloadConfig::new(104857600))
  })
  .bind(("0.0.0.0", port))?
  .run()
  .await
}
