use actix_web::HttpResponse;
use actix_web::web::{Data, Json, Path};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::api_token::{generate_token, hash_token, token_prefix};
use crate::auth::extractors::{OrgAdmin, OrgMember};
use crate::db;
use crate::models::api_token::{CreateTokenRequest, CreateTokenResponse};

pub async fn create_token(
  admin: OrgAdmin,
  pool: Data<PgPool>,
  body: Json<CreateTokenRequest>,
) -> HttpResponse {
  if body.name.is_empty() {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": "Token name is required"
    }));
  }

  // Check max_tokens limit
  if let Ok(Some(org)) = db::organizations::find_by_id(&pool, admin.org_id).await
    && let Some(max_tokens) = org.max_tokens
  {
    let current_count = db::api_tokens::count_by_org(&pool, admin.org_id)
      .await
      .unwrap_or(0);
    if current_count >= max_tokens as i64 {
      return HttpResponse::BadRequest().json(serde_json::json!({
        "error": format!("Token limit reached ({}/{})", current_count, max_tokens)
      }));
    }
  }

  let token = generate_token();
  let hash = hash_token(&token);
  let prefix = token_prefix(&token);
  let scopes = body
    .scopes
    .clone()
    .unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);

  const VALID_SCOPES: &[&str] = &["read", "write"];
  for scope in &scopes {
    if !VALID_SCOPES.contains(&scope.as_str()) {
      return HttpResponse::BadRequest().json(serde_json::json!({
        "error": format!("Invalid scope '{}'. Valid scopes: read, write", scope)
      }));
    }
  }

  let expires_at = body
    .expires_in_days
    .map(|days| Utc::now() + Duration::days(days));

  match db::api_tokens::create(
    &pool,
    admin.org_id,
    body.team_id,
    &body.name,
    &hash,
    &prefix,
    &scopes,
    admin.claims.sub,
    expires_at,
  )
  .await
  {
    Ok(db_token) => HttpResponse::Created().json(CreateTokenResponse {
      id: db_token.id,
      name: db_token.name,
      token,
      token_prefix: db_token.token_prefix,
      scopes: db_token.scopes,
      expires_at: db_token.expires_at,
      created_at: db_token.created_at,
    }),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to create token: {e}")
    })),
  }
}

pub async fn list_tokens(member: OrgMember, pool: Data<PgPool>) -> HttpResponse {
  match db::api_tokens::list_by_org(&pool, member.org_id).await {
    Ok(tokens) => HttpResponse::Ok().json(tokens),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to list tokens: {e}")
    })),
  }
}

pub async fn revoke_token(
  _admin: OrgAdmin,
  pool: Data<PgPool>,
  path: Path<(Uuid, Uuid)>,
) -> HttpResponse {
  let (_org_id, token_id) = path.into_inner();

  match db::api_tokens::revoke(&pool, token_id).await {
    Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "message": "Token revoked" })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to revoke token: {e}")
    })),
  }
}
