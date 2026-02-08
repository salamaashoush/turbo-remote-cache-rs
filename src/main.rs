use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{
  App, HttpRequest, HttpResponse, HttpServer,
  middleware::{Compress, DefaultHeaders},
  web::{self, Data, PayloadConfig},
};
use std::{env::args, path::Path, sync::Arc, time::Duration};
use tracing::{info, warn};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

use crate::config::{Config, get_port};
use crate::handlers::{
  admin, analytics, artifacts, cache, organizations, teams, tokens, turbo_api, turborepo,
};
use crate::storage::StorageStore;

pub mod auth;
pub mod config;
pub mod db;
pub mod email;
pub mod handlers;
pub mod helpers;
pub mod jobs;
pub mod models;
pub mod storage;

use handlers::auth as auth_handlers;

async fn bootstrap_super_admin(pool: &sqlx::PgPool, config: &Config) {
  let (email, password) = match (&config.super_admin_email, &config.super_admin_password) {
    (Some(e), Some(p)) => (e.as_str(), p.as_str()),
    _ => {
      warn!(
        "SUPER_ADMIN_EMAIL and/or SUPER_ADMIN_PASSWORD not set — skipping super admin bootstrap"
      );
      return;
    }
  };

  match db::users::find_by_email(pool, email).await {
    Ok(Some(user)) if user.role == "super_admin" => {
      info!("Super admin '{}' already exists", email);
    }
    Ok(Some(user)) => match db::users::update_role(pool, user.id, "super_admin").await {
      Ok(_) => info!("Upgraded existing user '{}' to super_admin", email),
      Err(e) => warn!("Failed to upgrade user '{}' to super_admin: {}", email, e),
    },
    Ok(None) => {
      let password_hash = match auth::password::hash_password(password) {
        Ok(h) => h,
        Err(e) => {
          warn!("Failed to hash super admin password: {}", e);
          return;
        }
      };
      match db::users::create_user(pool, email, &password_hash, "Admin", "super_admin").await {
        Ok(_) => info!("Created super admin user '{}'", email),
        Err(e) => warn!("Failed to create super admin user '{}': {}", email, e),
      }
    }
    Err(e) => {
      warn!("Failed to look up super admin user '{}': {}", email, e);
    }
  }
}

/// SPA fallback: serve index.html for any unmatched route (client-side routing)
async fn spa_fallback(_req: HttpRequest) -> actix_web::Result<actix_files::NamedFile> {
  Ok(actix_files::NamedFile::open("./frontend/dist/index.html")?)
}

fn configure_management_api(cfg: &mut web::ServiceConfig) {
  // Rate limiter for auth endpoints: 10 requests per 60s per IP
  let auth_rate_limit = GovernorConfigBuilder::default()
    .seconds_per_request(6)
    .burst_size(10)
    .finish()
    .expect("Failed to build auth rate limiter");

  cfg.service(
    web::scope("/api/v1")
      // Auth routes (no auth required for register/login)
      .service(
        web::scope("/auth")
          .wrap(Governor::new(&auth_rate_limit))
          .route("/register", web::post().to(auth_handlers::register))
          .route("/login", web::post().to(auth_handlers::login))
          .route("/refresh", web::post().to(auth_handlers::refresh))
          .route("/logout", web::post().to(auth_handlers::logout))
          .route("/me", web::get().to(auth_handlers::me))
          .route("/me", web::patch().to(auth_handlers::update_profile))
          .route("/verify-email", web::post().to(auth_handlers::verify_email))
          .route(
            "/forgot-password",
            web::post().to(auth_handlers::forgot_password),
          )
          .route(
            "/reset-password",
            web::post().to(auth_handlers::reset_password),
          )
          // 2FA routes
          .route(
            "/2fa/verify-login",
            web::post().to(auth_handlers::verify_twofa_login),
          )
          .route(
            "/2fa/status",
            web::get().to(auth_handlers::get_twofa_status),
          )
          .route("/2fa/totp/setup", web::post().to(auth_handlers::setup_totp))
          .route(
            "/2fa/totp/confirm",
            web::post().to(auth_handlers::confirm_totp),
          )
          .route(
            "/2fa/email/enable",
            web::post().to(auth_handlers::enable_email_twofa),
          )
          .route("/2fa/disable", web::post().to(auth_handlers::disable_twofa))
          .route(
            "/2fa/recovery/regenerate",
            web::post().to(auth_handlers::regenerate_recovery_codes),
          ),
      )
      // Organization routes
      .service(
        web::scope("/orgs")
          .route("", web::get().to(organizations::list_orgs))
          .route("/{org_id}", web::get().to(organizations::get_org))
          .route("/{org_id}", web::patch().to(organizations::update_org))
          .route("/{org_id}", web::delete().to(organizations::delete_org))
          // Org members
          .route(
            "/{org_id}/members",
            web::get().to(organizations::list_members),
          )
          .route(
            "/{org_id}/members",
            web::post().to(organizations::add_member),
          )
          .route(
            "/{org_id}/members/{user_id}",
            web::patch().to(organizations::update_member_role),
          )
          .route(
            "/{org_id}/members/{user_id}",
            web::delete().to(organizations::remove_member),
          )
          // Teams
          .route("/{org_id}/teams", web::post().to(teams::create_team))
          .route("/{org_id}/teams", web::get().to(teams::list_teams))
          .route("/{org_id}/teams/{team_id}", web::get().to(teams::get_team))
          .route(
            "/{org_id}/teams/{team_id}",
            web::patch().to(teams::update_team),
          )
          .route(
            "/{org_id}/teams/{team_id}",
            web::delete().to(teams::delete_team),
          )
          .route(
            "/{org_id}/teams/{team_id}/members",
            web::get().to(teams::list_team_members),
          )
          .route(
            "/{org_id}/teams/{team_id}/members",
            web::post().to(teams::add_team_member),
          )
          .route(
            "/{org_id}/teams/{team_id}/members/{user_id}",
            web::delete().to(teams::remove_team_member),
          )
          // API Tokens
          .route("/{org_id}/tokens", web::post().to(tokens::create_token))
          .route("/{org_id}/tokens", web::get().to(tokens::list_tokens))
          .route(
            "/{org_id}/tokens/{token_id}",
            web::delete().to(tokens::revoke_token),
          )
          // Analytics
          .route(
            "/{org_id}/analytics/overview",
            web::get().to(analytics::overview),
          )
          .route(
            "/{org_id}/analytics/timeline",
            web::get().to(analytics::timeline),
          )
          .route(
            "/{org_id}/analytics/teams",
            web::get().to(analytics::teams_breakdown),
          )
          // Cache management
          .route(
            "/{org_id}/cache/artifacts",
            web::get().to(cache::list_artifacts),
          )
          .route(
            "/{org_id}/cache/artifacts/{hash}",
            web::delete().to(cache::purge_artifact),
          )
          .route("/{org_id}/cache/purge", web::post().to(cache::purge_all)),
      )
      // Admin routes
      .service(
        web::scope("/admin")
          .route("/orgs", web::get().to(admin::list_orgs))
          .route("/orgs", web::post().to(admin::create_org))
          .route("/orgs/{org_id}", web::get().to(admin::get_org))
          .route("/orgs/{org_id}", web::delete().to(admin::delete_org))
          .route(
            "/orgs/{org_id}/limits",
            web::patch().to(admin::update_org_limits),
          )
          .route(
            "/orgs/{org_id}/transfer",
            web::post().to(admin::transfer_org_ownership),
          )
          .route(
            "/orgs/{org_id}/purge-cache",
            web::post().to(admin::purge_org_cache),
          )
          .route("/users", web::get().to(admin::list_users))
          .route("/users/{user_id}", web::get().to(admin::get_user))
          .route("/users/{user_id}", web::patch().to(admin::update_user))
          .route(
            "/users/{user_id}",
            web::delete().to(admin::delete_user_handler),
          )
          .route(
            "/users/{user_id}/activate",
            web::post().to(admin::activate_user),
          )
          .route(
            "/users/{user_id}/deactivate",
            web::post().to(admin::deactivate_user),
          )
          .route(
            "/users/{user_id}/force-logout",
            web::post().to(admin::force_logout),
          )
          .route(
            "/users/{user_id}/reset-password",
            web::post().to(admin::admin_password_reset),
          )
          .route(
            "/users/{user_id}/orgs",
            web::post().to(admin::add_user_to_org),
          )
          .route(
            "/users/{user_id}/orgs/{org_id}",
            web::delete().to(admin::remove_user_from_org),
          )
          .route("/stats", web::get().to(admin::platform_stats))
          .route(
            "/analytics/overview",
            web::get().to(admin::platform_overview),
          )
          .route(
            "/analytics/timeline",
            web::get().to(admin::platform_timeline),
          )
          .route(
            "/analytics/orgs",
            web::get().to(admin::platform_org_breakdown),
          )
          .route("/email/status", web::get().to(admin::email_status)),
      ),
  );
}

/// Health check: liveness probe
async fn health() -> HttpResponse {
  HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

/// Health check: readiness probe (checks DB connectivity if in platform mode)
async fn health_ready(pool: Option<Data<sqlx::PgPool>>) -> HttpResponse {
  if let Some(pool) = pool {
    match pool.acquire().await {
      Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })),
      Err(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
        "status": "unavailable",
        "reason": "database connection failed"
      })),
    }
  } else {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
  }
}

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

  // Create storage store once at startup (fail fast with clear message)
  let storage =
    StorageStore::new(&config).expect("Failed to initialize storage provider — check config");
  let storage_data = Data::new(storage);

  // Set up database pool if in platform mode
  let pool: Option<sqlx::PgPool> = if let Some(ref database_url) = config.database_url {
    info!("Platform mode: connecting to PostgreSQL...");
    let pool = sqlx::postgres::PgPoolOptions::new()
      .max_connections(config.db_max_connections)
      .min_connections(config.db_min_connections)
      .acquire_timeout(Duration::from_secs(5))
      .connect(database_url)
      .await
      .expect("Failed to connect to PostgreSQL");
    sqlx::migrate!()
      .run(&pool)
      .await
      .expect("Failed to run database migrations");
    info!("Database migrations completed");

    // Bootstrap super admin from environment variables
    bootstrap_super_admin(&pool, &config).await;

    // Spawn background cleanup jobs
    jobs::cleanup::spawn_cleanup_jobs(pool.clone());

    Some(pool)
  } else {
    info!("Legacy mode: no database configured, using env-var tokens only");
    None
  };

  // Build mailer if SMTP is configured
  let mailer: Option<email::Mailer> = config.smtp.as_ref().map(|smtp_config| {
    info!(
      "SMTP configured: {} (port {})",
      smtp_config.host, smtp_config.port
    );
    email::Mailer::new(smtp_config).expect("Failed to build SMTP mailer")
  });
  if mailer.is_none() {
    info!("SMTP not configured — email features disabled");
  }

  info!("Starting HTTP server at http://localhost:{}", port);

  let pool_clone = pool.clone();
  let mailer_clone = mailer.clone();
  // Create and Start the HTTP server
  HttpServer::new(move || {
    // Build CORS policy
    let cors = if config.allowed_origins.is_empty() {
      Cors::default()
        .allow_any_origin()
        .allow_any_header()
        .allow_any_method()
    } else {
      let mut cors = Cors::default();
      for origin in &config.allowed_origins {
        cors = cors.allowed_origin(origin);
      }
      cors
        .allow_any_header()
        .allow_any_method()
        .supports_credentials()
    };

    let mut app = App::new()
      .wrap(TracingLogger::default())
      .wrap(Compress::default())
      .wrap(cors)
      .wrap(
        DefaultHeaders::new()
          .add(("X-Content-Type-Options", "nosniff"))
          .add(("X-Frame-Options", "DENY"))
          .add(("X-XSS-Protection", "1; mode=block"))
          .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
          .add((
            "Permissions-Policy",
            "camera=(), microphone=(), geolocation=()",
          )),
      )
      // Health check endpoints (no auth)
      .route("/health", web::get().to(health))
      .route("/health/ready", web::get().to(health_ready))
      .app_data(Data::new(config.clone()))
      .app_data(Data::new(mailer_clone.clone()))
      .app_data(storage_data.clone())
      .configure(artifacts::configure)
      .app_data(PayloadConfig::new(104857600));

    // Add DB pool and management API routes if in platform mode
    if let Some(ref pool) = pool_clone {
      app = app
        .app_data(Data::new(pool.clone()))
        .configure(configure_management_api)
        .configure(turborepo::configure)
        // Vercel-compatible endpoints for turbo CLI (login/link/logout)
        .route("/v2/user", web::get().to(turbo_api::get_user))
        .route("/v2/teams", web::get().to(turbo_api::list_teams))
        .route(
          "/v5/user/tokens/current",
          web::get().to(turbo_api::get_current_token),
        )
        .route(
          "/v3/user/tokens/current",
          web::delete().to(turbo_api::delete_current_token),
        );
    }

    // Serve frontend static files (if dist directory exists)
    if std::path::Path::new("./frontend/dist").exists() {
      app = app
        .service(actix_files::Files::new("/assets", "./frontend/dist/assets"))
        .default_service(web::to(spa_fallback));
    }

    app
  })
  .shutdown_timeout(30)
  .bind(("0.0.0.0", port))?
  .run()
  .await
}
