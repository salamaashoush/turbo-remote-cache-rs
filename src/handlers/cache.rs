use actix_web::HttpResponse;
use actix_web::web::{Data, Path, Query};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::extractors::{OrgAdmin, OrgMember};
use crate::db;
use crate::storage::StorageStore;

#[derive(Deserialize)]
pub struct ListQuery {
  pub limit: Option<i64>,
  pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct PurgeQuery {
  pub team_id: Option<Uuid>,
}

pub async fn list_artifacts(
  member: OrgMember,
  pool: Data<PgPool>,
  query: Query<ListQuery>,
) -> HttpResponse {
  let limit = query.limit.unwrap_or(50).min(100);
  let offset = query.offset.unwrap_or(0);

  match db::cache_events::list_artifacts(&pool, member.org_id, limit, offset).await {
    Ok(artifacts) => HttpResponse::Ok().json(artifacts),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to list artifacts: {e}")
    })),
  }
}

pub async fn purge_artifact(
  _admin: OrgAdmin,
  storage: Data<StorageStore>,
  path: Path<(Uuid, String)>,
) -> HttpResponse {
  let (org_id, hash) = path.into_inner();
  let artifact_path = format!("{}/{}", org_id, hash);

  match storage.delete(&artifact_path).await {
    Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "message": "Artifact purged" })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to purge artifact: {e}")
    })),
  }
}

pub async fn purge_all(
  admin: OrgAdmin,
  storage: Data<StorageStore>,
  query: Query<PurgeQuery>,
) -> HttpResponse {
  let prefix = match query.team_id {
    Some(team_id) => format!("{}/{}/", admin.org_id, team_id),
    None => format!("{}/", admin.org_id),
  };

  match storage.delete_prefix(&prefix).await {
    Ok(count) => HttpResponse::Ok().json(serde_json::json!({
      "message": format!("Purged {} artifacts", count),
      "count": count
    })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to purge: {e}")
    })),
  }
}
