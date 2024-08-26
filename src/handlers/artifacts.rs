use crate::{
  auth::Auth,
  config::Config,
  helpers::{
    artifact_params_or_400, exists_cached_artifact, get_artifact_path, not_found, GetArtifactQuery,
  },
  storage::StorageStore,
};
use actix_web::{
  web::{get, head, post, put, resource, scope, Bytes, Data, Path, Query, ServiceConfig},
  HttpResponse, Responder,
};
use log::info;
use serde::Serialize;

#[derive(Serialize)]
pub struct Status {
  status: String,
}

#[derive(Serialize)]
struct PutArtifactResponse {
  urls: Vec<String>,
}

async fn post_artifacts_events() -> impl Responder {
  info!("Artifacts events received");
  HttpResponse::Ok()
    .content_type("application/json")
    .body("{}")
}

async fn get_status() -> impl Responder {
  let obj = Status {
    status: "enabled".to_string(),
  };
  info!("Status retrieved");
  HttpResponse::Ok()
    .content_type("application/json")
    .json(obj)
}

async fn head_artifact(
  path: Path<String>,
  query: Query<GetArtifactQuery>,
  storage: Data<StorageStore>,
) -> impl Responder {
  let (id, team_id) = match artifact_params_or_400(path, query) {
    Ok((id, team_id)) => (id, team_id),
    Err(e) => return e,
  };

  if exists_cached_artifact(&id, &team_id, &storage)
    .await
    .is_ok()
  {
    info!("Artifact {} exists", id);
    HttpResponse::Ok()
      .content_type("application/json")
      .body("true")
  } else {
    not_found("Artifact not found".to_string())
  }
}
async fn get_artifact(
  path: Path<String>,
  query: Query<GetArtifactQuery>,
  storage: Data<StorageStore>,
) -> impl Responder {
  let (id, team_id) = match artifact_params_or_400(path, query) {
    Ok((id, team_id)) => (id, team_id),
    Err(e) => return e,
  };
  if exists_cached_artifact(&id, &team_id, &storage)
    .await
    .is_ok()
  {
    let path: String = get_artifact_path(&id, &team_id);
    let data = storage.get(&path).await.unwrap();
    info!("Artifact {} retrieved from {}", id, path);
    HttpResponse::Ok()
      .content_type("application/octet-stream")
      .body(data)
  } else {
    not_found("Artifact not found".to_string())
  }
}

async fn put_artifact(
  path: Path<String>,
  query: Query<GetArtifactQuery>,
  body: Bytes,
  storage: Data<StorageStore>,
) -> impl Responder {
  let (id, team_id) = match artifact_params_or_400(path, query) {
    Ok((id, team_id)) => (id, team_id),
    Err(e) => return e,
  };
  // store artifact
  let path = get_artifact_path(&id, &team_id);
  let _ = storage.put(&path, body).await;
  info!("Artifact {} stored in {}", id, path);
  HttpResponse::Ok()
    .content_type("application/json")
    .json(PutArtifactResponse { urls: vec![path] })
}

pub fn configure(config: &Config) -> impl FnOnce(&mut ServiceConfig) + '_ {
  let c = |cfg: &mut ServiceConfig| {
    cfg.service(
      scope("/v8/artifacts")
        .route("/status", get().to(get_status))
        .service(
          scope("")
            .wrap(Auth)
            .route("/events", post().to(post_artifacts_events))
            .service(
              resource("/{id}")
                .route(get().to(get_artifact))
                .route(head().to(head_artifact))
                .route(put().to(put_artifact))
                .app_data(Data::new(StorageStore::new(config))),
            ),
        ),
    );
  };
  c
}

#[cfg(test)]
mod artifacts_tests {
  use core::str;
  use std::sync::Arc;

  use super::*;
  use crate::config::{Config, StorageProvider};
  use actix_web::{
    http::{header::ContentType, Method},
    test, App,
  };

  #[actix_web::test]
  async fn test_get_status() {
    let config = Arc::new(Config::default());
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .configure(configure(&config)),
    )
    .await;
    let req = test::TestRequest::get()
      .insert_header(ContentType::json())
      .uri("/v8/artifacts/status")
      .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body = test::read_body(resp).await;
    assert_eq!(body, r#"{"status":"enabled"}"#);
  }

  #[actix_web::test]
  async fn test_artifacts_unauthorized() {
    let config = Arc::new(Config::default());
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .configure(configure(&config)),
    )
    .await;
    let req = test::TestRequest::default()
      .method(Method::HEAD)
      .uri("/v8/artifacts/123?team_id=test")
      .insert_header(ContentType::json())
      .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
    assert_eq!(
      test::read_body(resp).await,
      r#"{"statusCode":401,"error":"Unauthorized","message":"Missing Authorization header"}"#
    );
  }

  #[actix_web::test]
  async fn test_artifacts_authorized() {
    let config = Arc::new(Config::default().with_turbo_tokens(vec!["test".to_string()]));
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .configure(configure(&config)),
    )
    .await;
    let req = test::TestRequest::default()
      .method(Method::HEAD)
      .uri("/v8/artifacts/123?teamId=test")
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
    assert_eq!(
      test::read_body(resp).await,
      r#"{"statusCode":404,"error":"Not Found","message":"Artifact not found"}"#
    );
  }

  #[actix_web::test]
  async fn test_artifacts_without_team_param() {
    let config = Arc::new(Config::default().with_turbo_tokens(vec!["test".to_string()]));
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .configure(configure(&config)),
    )
    .await;
    let req = test::TestRequest::default()
      .method(Method::HEAD)
      .uri("/v8/artifacts/123")
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
  }

  #[actix_web::test]
  async fn test_artifacts_head_ok() {
    let config = Arc::new(Config::default().with_turbo_tokens(vec!["test".to_string()]));
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .configure(configure(&config)),
    )
    .await;
    let put_req = test::TestRequest::default()
      .method(Method::PUT)
      .uri("/v8/artifacts/123?teamId=test")
      .set_payload(Bytes::from_static(b"test"))
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .to_request();
    let put_resp = test::call_service(&app, put_req).await;
    assert_eq!(put_resp.status(), 200);

    let head_req = test::TestRequest::default()
      .method(Method::HEAD)
      .uri("/v8/artifacts/123?teamId=test")
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .to_request();

    let head_resp = test::call_service(&app, head_req).await;
    assert_eq!(head_resp.status(), 200);
  }

  #[actix_web::test]
  async fn test_artifacts_get_ok() {
    let config = Arc::new(Config::default().with_turbo_tokens(vec!["test".to_string()]));
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .configure(configure(&config)),
    )
    .await;
    let put_req = test::TestRequest::default()
      .method(Method::PUT)
      .uri("/v8/artifacts/123?teamId=test")
      .set_payload(Bytes::from_static(b"test"))
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .to_request();
    let put_resp = test::call_service(&app, put_req).await;
    assert_eq!(put_resp.status(), 200);

    let get_req = test::TestRequest::default()
      .method(Method::GET)
      .uri("/v8/artifacts/123?teamId=test")
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);
    let body = test::read_body(get_resp).await;
    assert_eq!(str::from_utf8(&body).unwrap(), "test");
  }

  #[actix_web::test]
  async fn test_artifacts_put_ok() {
    let config = Arc::new(Config::default().with_turbo_tokens(vec!["test".to_string()]));
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .configure(configure(&config)),
    )
    .await;
    let put_req = test::TestRequest::default()
      .method(Method::PUT)
      .uri("/v8/artifacts/123?teamId=test")
      .set_payload(Bytes::from_static(b"test"))
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .to_request();
    let put_resp = test::call_service(&app, put_req).await;
    assert_eq!(put_resp.status(), 200);
    let body = test::read_body(put_resp).await;
    assert_eq!(str::from_utf8(&body).unwrap(), r#"{"urls":["test/123"]}"#);
  }

  #[actix_web::test]
  async fn test_artifacts_with_file_provider() {
    let config = Arc::new(
      Config::default()
        .with_turbo_tokens(vec!["test".to_string()])
        .with_storage_provider(StorageProvider::File)
        .with_fs_cache_path("test_files".to_string()),
    );
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .configure(configure(&config)),
    )
    .await;
    let put_req = test::TestRequest::default()
      .method(Method::PUT)
      .uri("/v8/artifacts/123?teamId=test")
      .set_payload(Bytes::from_static(b"test"))
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .to_request();
    let put_resp = test::call_service(&app, put_req).await;
    assert_eq!(put_resp.status(), 200);
    let body = test::read_body(put_resp).await;
    assert_eq!(str::from_utf8(&body).unwrap(), r#"{"urls":["test/123"]}"#);

    let get_req = test::TestRequest::default()
      .method(Method::GET)
      .uri("/v8/artifacts/123?teamId=test")
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);
    let body = test::read_body(get_resp).await;
    assert_eq!(str::from_utf8(&body).unwrap(), "test");
  }
}
