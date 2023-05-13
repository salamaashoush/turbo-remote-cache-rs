use crate::{
    helpers::{artifact_params_or_400, internal_server_error, not_found, GetArtifactQuery},
    storage::local::{create_cached_artifact, exists_cached_artifact, get_cached_artifact},
};
use actix_web::{
    web::{get, head, post, put, resource, scope, Bytes, Path, Query, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
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

pub async fn head_artifact(path: Path<String>, query: Query<GetArtifactQuery>) -> impl Responder {
    let (id, team_id) = match artifact_params_or_400(path, query) {
        Ok((id, team_id)) => (id, team_id),
        Err(e) => return e,
    };

    if exists_cached_artifact(id, team_id).is_ok() {
        HttpResponse::Ok()
            .content_type("application/json")
            .body("true")
    } else {
        not_found("Artifact not found".to_string())
    }
}
pub async fn get_artifact(
    req: HttpRequest,
    path: Path<String>,
    query: Query<GetArtifactQuery>,
) -> impl Responder {
    let (id, team_id) = match artifact_params_or_400(path, query) {
        Ok((id, team_id)) => (id, team_id),
        Err(e) => return e,
    };

    if exists_cached_artifact(id.clone(), team_id.clone()).is_ok() {
        let artifact = get_cached_artifact(id, team_id).await.unwrap();

        artifact.into_response(&req)
    } else {
        not_found("Artifact not found".to_string())
    }
}

pub async fn put_artifact(
    path: Path<String>,
    query: Query<GetArtifactQuery>,
    body: Bytes,
) -> impl Responder {
    let (id, team_id) = match artifact_params_or_400(path, query) {
        Ok((id, team_id)) => (id, team_id),
        Err(e) => return e,
    };
    // store artifact
    match create_cached_artifact(id.clone(), team_id.clone(), body) {
        Ok(_) => (),
        Err(message) => return internal_server_error(message),
    };
    HttpResponse::Accepted()
        .content_type("application/json")
        .json(PutArtifactResponse {
            urls: vec![format!("{}/{}", team_id, id)],
        })
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
