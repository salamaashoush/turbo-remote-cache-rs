use actix_web::HttpResponse;
use actix_web::web::{Data, Json, Path};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::extractors::{OrgAdmin, OrgMember};
use crate::db;
use crate::models::team::{AddTeamMemberRequest, CreateTeamRequest, UpdateTeamRequest};

pub async fn create_team(
  admin: OrgAdmin,
  pool: Data<PgPool>,
  body: Json<CreateTeamRequest>,
) -> HttpResponse {
  if body.name.is_empty() || body.slug.is_empty() {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": "Name and slug are required"
    }));
  }

  match db::teams::create(&pool, admin.org_id, &body.name, &body.slug).await {
    Ok(team) => HttpResponse::Created().json(team),
    Err(e) => HttpResponse::Conflict().json(serde_json::json!({
      "error": format!("Failed to create team: {e}")
    })),
  }
}

pub async fn list_teams(member: OrgMember, pool: Data<PgPool>) -> HttpResponse {
  match db::teams::list_by_org(&pool, member.org_id).await {
    Ok(teams) => HttpResponse::Ok().json(teams),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to list teams: {e}")
    })),
  }
}

pub async fn get_team(
  _member: OrgMember,
  pool: Data<PgPool>,
  path: Path<(Uuid, Uuid)>,
) -> HttpResponse {
  let (_org_id, team_id) = path.into_inner();

  match db::teams::find_by_id(&pool, team_id).await {
    Ok(Some(team)) => HttpResponse::Ok().json(team),
    Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Team not found" })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Database error: {e}")
    })),
  }
}

pub async fn update_team(
  _admin: OrgAdmin,
  pool: Data<PgPool>,
  path: Path<(Uuid, Uuid)>,
  body: Json<UpdateTeamRequest>,
) -> HttpResponse {
  let (_org_id, team_id) = path.into_inner();

  match db::teams::update(&pool, team_id, body.name.as_deref(), body.slug.as_deref()).await {
    Ok(team) => HttpResponse::Ok().json(team),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to update team: {e}")
    })),
  }
}

pub async fn delete_team(
  _admin: OrgAdmin,
  pool: Data<PgPool>,
  path: Path<(Uuid, Uuid)>,
) -> HttpResponse {
  let (_org_id, team_id) = path.into_inner();

  match db::teams::delete(&pool, team_id).await {
    Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "message": "Team deleted" })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to delete team: {e}")
    })),
  }
}

pub async fn list_team_members(
  _member: OrgMember,
  pool: Data<PgPool>,
  path: Path<(Uuid, Uuid)>,
) -> HttpResponse {
  let (_org_id, team_id) = path.into_inner();

  match db::team_members::list_members(&pool, team_id).await {
    Ok(members) => HttpResponse::Ok().json(members),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to list team members: {e}")
    })),
  }
}

pub async fn add_team_member(
  _admin: OrgAdmin,
  pool: Data<PgPool>,
  path: Path<(Uuid, Uuid)>,
  body: Json<AddTeamMemberRequest>,
) -> HttpResponse {
  let (_org_id, team_id) = path.into_inner();
  let role = body.role.as_deref().unwrap_or("member");

  match db::team_members::add_member(&pool, team_id, body.user_id, role).await {
    Ok(member) => HttpResponse::Created().json(member),
    Err(e) => HttpResponse::Conflict().json(serde_json::json!({
      "error": format!("Failed to add team member: {e}")
    })),
  }
}

pub async fn remove_team_member(
  _admin: OrgAdmin,
  pool: Data<PgPool>,
  path: Path<(Uuid, Uuid, Uuid)>,
) -> HttpResponse {
  let (_org_id, team_id, user_id) = path.into_inner();

  match db::team_members::remove_member(&pool, team_id, user_id).await {
    Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "message": "Team member removed" })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to remove team member: {e}")
    })),
  }
}
