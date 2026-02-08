use std::{
  future::{Ready, ready},
  sync::Arc,
};

use actix_web::{
  Error,
  body::EitherBody,
  dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
  web::Data,
};
use futures_util::future::LocalBoxFuture;

use crate::{
  config::Config,
  helpers::{bad_request, unauthorized},
};

type AppConfigData = Data<Arc<Config>>;

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type InitError = ();
  type Transform = AuthMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(AuthMiddleware { service }))
  }
}
pub struct AuthMiddleware<S> {
  service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, request: ServiceRequest) -> Self::Future {
    let turbo_tokens = request
      .app_data::<AppConfigData>()
      .map(|data| data.turbo_tokens.clone());

    let turbo_tokens = match turbo_tokens {
      Some(tokens) => tokens,
      None => {
        let (req, _pl) = request.into_parts();
        let response =
          bad_request("Missing TURBO_TOKENS in the environment".to_string()).map_into_right_body();
        return Box::pin(async { Ok(ServiceResponse::new(req, response)) });
      }
    };

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

    if !turbo_tokens.contains(&token) {
      let (req, _pl) = request.into_parts();
      let response = unauthorized("Invalid Turbo Token".to_string()).map_into_right_body();
      return Box::pin(async { Ok(ServiceResponse::new(req, response)) });
    }

    let res = self.service.call(request);
    Box::pin(async move { res.await.map(ServiceResponse::map_into_left_body) })
  }
}
