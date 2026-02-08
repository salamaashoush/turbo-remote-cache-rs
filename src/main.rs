use actix_cors::Cors;
use actix_web::{
  App, HttpServer,
  middleware::Compress,
  web::{Data, PayloadConfig},
};
use std::{env::args, path::Path, sync::Arc};
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

use crate::config::{Config, get_port};
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
  // Initialize tracing subscriber
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
    .init();
  let config = Arc::new(Config::from_env().expect("error loading config from environment"));
  let port = get_port();
  info!(
    "Using {} storage provider with bucket {} at {}",
    config.storage_provider, config.bucket_name, config.fs_cache_path
  );
  info!("Starting HTTP server at http://localhost:{}", port);
  // Create and Start the HTTP server
  HttpServer::new(move || {
    App::new()
      .wrap(TracingLogger::default())
      .wrap(Compress::default())
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
