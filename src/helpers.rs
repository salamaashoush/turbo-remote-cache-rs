use actix_web::{
    web::{Path, Query},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct BoomResponse {
    statusCode: i32,
    error: Option<String>,
    message: String,
}
pub fn bad_request(message: String) -> HttpResponse {
    let value = BoomResponse {
        statusCode: 400,
        error: Some("Bad Request".to_string()),
        message,
    };
    HttpResponse::BadRequest()
        .content_type("application/json")
        .json(value)
}

pub fn not_found(message: String) -> HttpResponse {
    let value = BoomResponse {
        statusCode: 404,
        error: Some("Not Found".to_string()),
        message,
    };
    HttpResponse::NotFound()
        .content_type("application/json")
        .json(value)
}

pub fn internal_server_error(message: String) -> HttpResponse {
    let value = BoomResponse {
        statusCode: 500,
        error: Some("Internal Server Error".to_string()),
        message,
    };
    HttpResponse::InternalServerError()
        .content_type("application/json")
        .json(value)
}

pub fn not_implemented(message: String) -> HttpResponse {
    let value = BoomResponse {
        statusCode: 501,
        error: Some("Not Implemented".to_string()),
        message,
    };
    HttpResponse::NotImplemented()
        .content_type("application/json")
        .json(value)
}

pub fn precondition_failed(message: String) -> HttpResponse {
    let value = BoomResponse {
        statusCode: 412,
        error: Some("Precondition Failed".to_string()),
        message,
    };
    HttpResponse::PreconditionFailed()
        .content_type("application/json")
        .json(value)
}

pub fn ok(message: String) -> HttpResponse {
    let value = BoomResponse {
        statusCode: 200,
        error: None,
        message,
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .json(value)
}

#[derive(Deserialize)]
pub struct GetArtifactQuery {
    teamId: Option<String>,
    slug: Option<String>,
}

pub fn artifact_params_or_400(
    path: Path<String>,
    query: Query<GetArtifactQuery>,
) -> Result<(String, String), HttpResponse> {
    let id = path.into_inner();
    let GetArtifactQuery { teamId, slug } = query.into_inner();

    if teamId.is_none() && slug.is_none() {
        return Err(bad_request(
            "querystring should have required property 'teamId'".to_string(),
        ));
    }

    let team_id = teamId.unwrap_or_else(|| slug.unwrap().to_string());

    Ok((id, team_id))
}
