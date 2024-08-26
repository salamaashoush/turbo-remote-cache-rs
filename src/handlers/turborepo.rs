use actix_web::{
  web::{get, scope, Query, ServiceConfig},
  HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Token {
  status: String,
}

#[derive(Deserialize)]
pub struct GetArtifactQuery {
  redirect_uri: String,
}
// turborepo/token
pub async fn get_token(query: Query<GetArtifactQuery>) -> impl Responder {
  let obj = Token {
    status: format!("redirect to {}, token", query.redirect_uri),
  };
  HttpResponse::Ok()
    .content_type("application/json")
    .json(obj)
}

pub fn configure(cfg: &mut ServiceConfig) {
  cfg.service(scope("/turborepo").route("/token", get().to(get_token)));
}
