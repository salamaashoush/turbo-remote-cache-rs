use crate::{
    helpers::{
        artifact_params_or_400, exists_cached_artifact, get_artifact_path, not_found,
        GetArtifactQuery,
    },
    storage::StorageStore,
};
use actix_web::{
    web::{get, head, post, put, resource, scope, Bytes, Data, Path, Query, ServiceConfig},
    HttpResponse, Responder,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct Status {
    status: String,
}

#[derive(Serialize)]
struct PutArtifactResponse {
    urls: Vec<String>,
}

pub async fn post_artifacts_events() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("{}")
}

pub async fn get_status() -> impl Responder {
    let obj = Status {
        status: "enabled".to_string(),
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .json(obj)
}

pub async fn head_artifact(
    path: Path<String>,
    query: Query<GetArtifactQuery>,
    storage: Data<StorageStore>,
) -> impl Responder {
    let (id, team_id) = match artifact_params_or_400(path, query) {
        Ok((id, team_id)) => (id, team_id),
        Err(e) => return e,
    };

    if exists_cached_artifact(id.clone(), team_id.clone(), &storage)
        .await
        .is_ok()
    {
        HttpResponse::Ok()
            .content_type("application/json")
            .body("true")
    } else {
        not_found("Artifact not found".to_string())
    }
}
pub async fn get_artifact(
    path: Path<String>,
    query: Query<GetArtifactQuery>,
    storage: Data<StorageStore>,
) -> impl Responder {
    let (id, team_id) = match artifact_params_or_400(path, query) {
        Ok((id, team_id)) => (id, team_id),
        Err(e) => return e,
    };
    println!("get_artifact before the check: {}", id);
    if exists_cached_artifact(id.clone(), team_id.clone(), &storage)
        .await
        .is_ok()
    {
        println!("get_artifact: {}", id);

        let path: String = get_artifact_path(id, team_id);
        let data = storage.get(&path).await.unwrap();
        HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(data)
    } else {
        not_found("Artifact not found".to_string())
    }
}

pub async fn put_artifact(
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
    let path = get_artifact_path(id, team_id);
    println!("put_artifact: {}", path);
    storage.put(&path, body).await;
    HttpResponse::Ok()
        .content_type("application/json")
        .json(PutArtifactResponse { urls: vec![path] })
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/artifacts")
            .route("/events", post().to(post_artifacts_events))
            .route("/status", get().to(get_status))
            .service(
                resource("/{id}")
                    .route(get().to(get_artifact))
                    .route(head().to(head_artifact))
                    .route(put().to(put_artifact)),
            ),
    );
}
