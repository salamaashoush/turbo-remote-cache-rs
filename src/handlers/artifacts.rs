use crate::{
  auth::{Auth, AuthInfo},
  helpers::{
    GetArtifactQuery, artifact_params_or_400, exists_cached_artifact, get_artifact_path,
    internal_server_error, not_found,
  },
  storage::StorageStore,
};
use actix_web::{
  HttpMessage, HttpRequest, HttpResponse, Responder,
  web::{Bytes, Data, Path, Query, ServiceConfig, get, head, post, put, resource, scope},
};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
struct TurboAnalyticsEvent {
  #[allow(dead_code)]
  source: Option<String>,
  event: String,
  hash: String,
  duration: Option<i64>,
  #[allow(dead_code)]
  #[serde(rename = "sessionId")]
  session_id: Option<String>,
}

/// Record a cache event in the background (fire-and-forget) for API token requests.
fn maybe_record_event(
  req: &HttpRequest,
  artifact_hash: &str,
  event_type: &str,
  size_bytes: i64,
  duration_ms: i32,
) {
  if let Some(AuthInfo::ApiToken {
    org_id, team_id, ..
  }) = req.extensions().get::<AuthInfo>().cloned()
  {
    let hash = artifact_hash.to_string();
    let event = event_type.to_string();
    if let Some(pool) = req.app_data::<Data<sqlx::PgPool>>() {
      let pool = pool.get_ref().clone();
      tokio::spawn(async move {
        let _ = crate::db::cache_events::record_event(
          &pool,
          org_id,
          team_id,
          &hash,
          &event,
          size_bytes,
          duration_ms,
        )
        .await;
      });
    }
  }
}

#[derive(Serialize)]
pub struct Status {
  status: String,
}

#[derive(Serialize)]
struct PutArtifactResponse {
  urls: Vec<String>,
}

/// Turbo CLI sends analytics events as a JSON array: [{source, event, hash, duration, sessionId}]
async fn post_artifacts_events(req: HttpRequest, body: Bytes) -> impl Responder {
  info!("Artifacts events received ({} bytes)", body.len());

  // Parse and record events if we have org context (API token auth)
  if let Some(AuthInfo::ApiToken {
    org_id, team_id, ..
  }) = req.extensions().get::<AuthInfo>().cloned()
    && let Some(pool) = req.app_data::<Data<sqlx::PgPool>>()
    && let Ok(events) = serde_json::from_slice::<Vec<TurboAnalyticsEvent>>(&body)
  {
    let pool = pool.get_ref().clone();
    tokio::spawn(async move {
      for event in events {
        let event_type = match event.event.as_str() {
          "HIT" | "hit" => "hit",
          "MISS" | "miss" => "miss",
          _ => continue,
        };
        let _ = crate::db::cache_events::record_event(
          &pool,
          org_id,
          team_id,
          &event.hash,
          event_type,
          0,
          event.duration.unwrap_or(0) as i32,
        )
        .await;
      }
    });
  }

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
  req: HttpRequest,
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
    maybe_record_event(&req, &id, "miss", 0, 0);
    not_found("Artifact not found".to_string())
  }
}
async fn get_artifact(
  req: HttpRequest,
  path: Path<String>,
  query: Query<GetArtifactQuery>,
  storage: Data<StorageStore>,
) -> HttpResponse {
  let (id, team_id) = match artifact_params_or_400(path, query) {
    Ok((id, team_id)) => (id, team_id),
    Err(e) => return e,
  };
  if exists_cached_artifact(&id, &team_id, &storage)
    .await
    .is_ok()
  {
    let artifact_path = get_artifact_path(&id, &team_id);
    let stream = storage.get_stream(&artifact_path);

    let mut response = HttpResponse::Ok();
    response.content_type("application/octet-stream");

    if let Some(tag) = storage.get_tag(&artifact_path).await {
      response.insert_header(("x-artifact-tag", tag));
    }

    let duration_ms = storage.get_duration(&artifact_path).await.unwrap_or(0);
    if duration_ms > 0 {
      response.insert_header(("x-artifact-duration", duration_ms.to_string()));
    }

    maybe_record_event(&req, &id, "hit", 0, duration_ms);
    info!("Artifact {} retrieved from {}", id, artifact_path);
    response.streaming(stream)
  } else {
    maybe_record_event(&req, &id, "miss", 0, 0);
    not_found("Artifact not found".to_string())
  }
}

async fn put_artifact(
  req: HttpRequest,
  path: Path<String>,
  query: Query<GetArtifactQuery>,
  body: Bytes,
  storage: Data<StorageStore>,
) -> HttpResponse {
  let (id, team_id) = match artifact_params_or_400(path, query) {
    Ok((id, team_id)) => (id, team_id),
    Err(e) => return e,
  };

  // Check cache size limit for platform mode (API token auth)
  let auth_info = req.extensions().get::<AuthInfo>().cloned();
  if let Some(AuthInfo::ApiToken { org_id, .. }) = auth_info
    && let Some(pool) = req.app_data::<Data<sqlx::PgPool>>()
    && let Ok(Some(org)) = crate::db::organizations::find_by_id(pool.get_ref(), org_id).await
    && let Some(limit) = org.cache_size_limit_bytes
  {
    let current = crate::db::cache_events::total_bytes_by_org(pool.get_ref(), org_id)
      .await
      .unwrap_or(0);
    if current + body.len() as i64 > limit {
      return HttpResponse::PayloadTooLarge().json(serde_json::json!({
        "error": "Cache size limit exceeded"
      }));
    }
  }

  let artifact_path = get_artifact_path(&id, &team_id);
  let body_len = body.len();
  if let Err(e) = storage.put(&artifact_path, body).await {
    return internal_server_error(format!("Failed to store artifact: {e}"));
  }

  if let Some(tag_value) = req.headers().get("x-artifact-tag")
    && let Ok(tag_str) = tag_value.to_str()
  {
    let _ = storage.put_tag(&artifact_path, tag_str).await;
  }

  // Store x-artifact-duration as sidecar for time-saved analytics
  let duration_ms = req
    .headers()
    .get("x-artifact-duration")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.parse::<i32>().ok())
    .unwrap_or(0);
  if duration_ms > 0 {
    let _ = storage.put_duration(&artifact_path, duration_ms).await;
  }

  let body_len = body_len as i64;
  maybe_record_event(&req, &id, "put", body_len, duration_ms);
  info!("Artifact {} stored in {}", id, artifact_path);
  HttpResponse::Ok()
    .content_type("application/json")
    .json(PutArtifactResponse {
      urls: vec![artifact_path],
    })
}

pub fn configure(cfg: &mut ServiceConfig) {
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
              .route(put().to(put_artifact)),
          ),
      ),
  );
}

#[cfg(test)]
mod artifacts_tests {
  use core::str;
  use std::sync::Arc;

  use super::*;
  use crate::config::{Config, StorageProvider};
  use crate::storage::StorageStore;
  use actix_web::{
    App,
    http::{Method, header::ContentType},
    test,
  };

  #[actix_web::test]
  async fn test_get_status() {
    let config = Arc::new(Config::default());
    let storage = StorageStore::new(&config).expect("storage");
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .app_data(Data::new(storage))
        .configure(configure),
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
    let storage = StorageStore::new(&config).expect("storage");
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .app_data(Data::new(storage))
        .configure(configure),
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
    let storage = StorageStore::new(&config).expect("storage");
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .app_data(Data::new(storage))
        .configure(configure),
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
    let storage = StorageStore::new(&config).expect("storage");
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .app_data(Data::new(storage))
        .configure(configure),
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
    let storage = StorageStore::new(&config).expect("storage");
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .app_data(Data::new(storage))
        .configure(configure),
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
    let storage = StorageStore::new(&config).expect("storage");
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .app_data(Data::new(storage))
        .configure(configure),
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
    let storage = StorageStore::new(&config).expect("storage");
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .app_data(Data::new(storage))
        .configure(configure),
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
    let storage = StorageStore::new(&config).expect("storage");
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .app_data(Data::new(storage))
        .configure(configure),
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

  #[actix_web::test]
  async fn test_artifacts_with_tag() {
    let config = Arc::new(Config::default().with_turbo_tokens(vec!["test".to_string()]));
    let storage = StorageStore::new(&config).expect("storage");
    let app = test::init_service(
      App::new()
        .app_data(Data::new(config.clone()))
        .app_data(Data::new(storage))
        .configure(configure),
    )
    .await;

    // PUT artifact with x-artifact-tag header
    let put_req = test::TestRequest::default()
      .method(Method::PUT)
      .uri("/v8/artifacts/tag123?teamId=test")
      .set_payload(Bytes::from_static(b"tagged-content"))
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .insert_header(("x-artifact-tag", "abc123def"))
      .to_request();
    let put_resp = test::call_service(&app, put_req).await;
    assert_eq!(put_resp.status(), 200);

    // GET artifact and verify x-artifact-tag is returned
    let get_req = test::TestRequest::default()
      .method(Method::GET)
      .uri("/v8/artifacts/tag123?teamId=test")
      .insert_header(ContentType::json())
      .insert_header(("Authorization", "Bearer test"))
      .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let tag_header = get_resp
      .headers()
      .get("x-artifact-tag")
      .expect("x-artifact-tag header missing")
      .to_str()
      .unwrap();
    assert_eq!(tag_header, "abc123def");

    let body = test::read_body(get_resp).await;
    assert_eq!(str::from_utf8(&body).unwrap(), "tagged-content");
  }
}
