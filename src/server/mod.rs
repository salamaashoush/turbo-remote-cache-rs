pub mod artifacts;
pub mod auth;
pub mod config;
pub mod helpers;
pub mod storage;

use crate::app::*;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{
    middleware::Logger,
    web::{scope, Data, PayloadConfig},
    App, HttpServer,
};
use leptos::*;
use leptos_actix::{generate_route_list, LeptosRoutes};
use std::{
    env::{args, set_var},
    path::Path,
};

use storage::StorageStore;

pub async fn run() -> std::io::Result<()> {
    let env_file = args().nth(1).unwrap_or(".env".to_string());
    dotenv::from_path(Path::new(&env_file)).ok();
    set_var("RUST_LOG", "info");
    set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let conf = get_configuration(None).await.unwrap();
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> });

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;
        App::new()
            .app_data(Data::new(StorageStore::new()))
            .service(scope("/v8").wrap(auth::Auth).configure(artifacts::config))
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                |cx| view! { cx, <App/> },
            )
            .service(Files::new("/", site_root))
            // .wrap(middleware::Compress::default())
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .app_data(PayloadConfig::new(104857600))
    })
    .bind(("0.0.0.0", config::get_port()))?
    .run()
    .await
}
