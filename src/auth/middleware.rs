use std::future::{Ready, ready};
use std::rc::Rc;
use std::sync::Arc;

use actix_web::HttpMessage;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::web::Data;
use actix_web::{Error, body::EitherBody};
use futures_util::future::LocalBoxFuture;
use sqlx::PgPool;

use crate::config::Config;
use crate::helpers::{bad_request, unauthorized};

use super::AuthInfo;
use super::api_token::hash_token;

type AppConfigData = Data<Arc<Config>>;

/// Auth middleware for the `/v8/artifacts/*` routes.
/// Supports dual auth: DB-backed API tokens and legacy env-var tokens.
pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type InitError = ();
  type Transform = AuthMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(AuthMiddleware {
      service: Rc::new(service),
    }))
  }
}

pub struct AuthMiddleware<S> {
  service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, request: ServiceRequest) -> Self::Future {
    let config = request
      .app_data::<AppConfigData>()
      .map(|data| data.as_ref().clone());

    let config = match config {
      Some(c) => c,
      None => {
        let (req, _pl) = request.into_parts();
        let response = bad_request("Missing configuration".to_string()).map_into_right_body();
        return Box::pin(async { Ok(ServiceResponse::new(req, response)) });
      }
    };

    // Extract the bearer token
    let auth_header = request.headers().get("Authorization");
    let token = match auth_header {
      None => {
        let (req, _pl) = request.into_parts();
        let response =
          unauthorized("Missing Authorization header".to_string()).map_into_right_body();
        return Box::pin(async { Ok(ServiceResponse::new(req, response)) });
      }
      Some(v) => {
        let header_str = match v.to_str() {
          Ok(s) => s,
          Err(_) => {
            let (req, _pl) = request.into_parts();
            let response =
              bad_request("Invalid Authorization header".to_string()).map_into_right_body();
            return Box::pin(async { Ok(ServiceResponse::new(req, response)) });
          }
        };
        match header_str.strip_prefix("Bearer ") {
          Some(token) => token.to_string(),
          None => {
            let (req, _pl) = request.into_parts();
            let response = bad_request("Authorization header must use Bearer scheme".to_string())
              .map_into_right_body();
            return Box::pin(async { Ok(ServiceResponse::new(req, response)) });
          }
        }
      }
    };

    let pool = request.app_data::<Data<PgPool>>().cloned();
    let service = self.service.clone();

    Box::pin(async move {
      // Try DB-backed API token lookup first
      if let Some(pool) = &pool {
        let token_hash = hash_token(&token);
        if let Ok(Some(db_token)) = crate::db::api_tokens::find_by_hash(pool, &token_hash).await {
          let is_valid = db_token
            .expires_at
            .map(|exp| exp > chrono::Utc::now())
            .unwrap_or(true);

          if is_valid {
            // Fire-and-forget: update last_used_at
            let pool_bg = pool.clone().into_inner();
            let token_id = db_token.id;
            tokio::spawn(async move {
              let _ = crate::db::api_tokens::update_last_used(&pool_bg, token_id).await;
            });

            // Attach AuthInfo to request extensions
            request.extensions_mut().insert(AuthInfo::ApiToken {
              token_id: db_token.id,
              org_id: db_token.org_id,
              team_id: db_token.team_id,
              scopes: db_token.scopes.clone(),
            });

            return service
              .call(request)
              .await
              .map(ServiceResponse::map_into_left_body);
          }
        }
      }

      // Fallback: legacy token check
      if config.legacy_tokens_enabled && config.turbo_tokens.contains(&token) {
        request.extensions_mut().insert(AuthInfo::LegacyToken);
        return service
          .call(request)
          .await
          .map(ServiceResponse::map_into_left_body);
      }

      // No valid auth
      let (req, _pl) = request.into_parts();
      let response = unauthorized("Invalid token".to_string()).map_into_right_body();
      Ok(ServiceResponse::new(req, response))
    })
  }
}
