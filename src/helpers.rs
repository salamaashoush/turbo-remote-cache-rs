use actix_web::{
    web::{Data, Path, Query},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

use crate::storage::StorageStore;

#[derive(Serialize)]
pub struct BoomResponse {
    #[serde(rename = "statusCode")]
    status_code: i32,
    error: Option<String>,
    message: String,
}
pub fn bad_request(message: String) -> HttpResponse {
    let value = BoomResponse {
        status_code: 400,
        error: Some("Bad Request".to_string()),
        message,
    };
    HttpResponse::BadRequest()
        .content_type("application/json")
        .json(value)
}

pub fn not_found(message: String) -> HttpResponse {
    let value = BoomResponse {
        status_code: 404,
        error: Some("Not Found".to_string()),
        message,
    };
    HttpResponse::NotFound()
        .content_type("application/json")
        .json(value)
}

pub fn internal_server_error(message: String) -> HttpResponse {
    let value = BoomResponse {
        status_code: 500,
        error: Some("Internal Server Error".to_string()),
        message,
    };
    HttpResponse::InternalServerError()
        .content_type("application/json")
        .json(value)
}

pub fn not_implemented(message: String) -> HttpResponse {
    let value = BoomResponse {
        status_code: 501,
        error: Some("Not Implemented".to_string()),
        message,
    };
    HttpResponse::NotImplemented()
        .content_type("application/json")
        .json(value)
}

pub fn precondition_failed(message: String) -> HttpResponse {
    let value = BoomResponse {
        status_code: 412,
        error: Some("Precondition Failed".to_string()),
        message,
    };
    HttpResponse::PreconditionFailed()
        .content_type("application/json")
        .json(value)
}

pub fn ok(message: String) -> HttpResponse {
    let value = BoomResponse {
        status_code: 200,
        error: None,
        message,
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .json(value)
}

#[derive(Deserialize)]
pub struct GetArtifactQuery {
    #[serde(rename = "teamId")]
    team_id: Option<String>,
    slug: Option<String>,
}

pub fn artifact_params_or_400(
    path: Path<String>,
    query: Query<GetArtifactQuery>,
) -> Result<(String, String), HttpResponse> {
    let id = path.into_inner();
    let GetArtifactQuery { team_id, slug } = query.into_inner();

    if team_id.is_none() && slug.is_none() {
        return Err(bad_request(
            "querystring should have required property 'teamId'".to_string(),
        ));
    }

    let team_id = team_id.unwrap_or_else(|| slug.unwrap().to_string());

    Ok((id, team_id))
}

pub fn get_artifact_path(artifact_id: String, team_id: String) -> String {
    format!("{}/{}", team_id, artifact_id)
}

pub async fn exists_cached_artifact(
    artifact_id: String,
    team_id: String,
    storage: &Data<StorageStore>,
) -> Result<bool, String> {
    let artifact_path = get_artifact_path(artifact_id, team_id);
    if !storage.exists(&artifact_path).await {
        return Err(format!("Artifact {} doesn't exist.", artifact_path));
    }
    Ok(true)
}
