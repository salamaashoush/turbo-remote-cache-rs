//! Endpoints that the Turbo CLI expects for `turbo login`, `turbo link`, `turbo logout`.
//! These mimic a subset of the Vercel API that Turborepo is hard-coded to call.
//!
//! Supports both JWT (dashboard) and API token (`trcs_...`) authentication,
//! since the Turbo CLI uses API tokens via `TURBO_TOKEN` / `turbo login`.

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Query};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::api_token::hash_token;
use crate::auth::jwt::validate_token;
use crate::config::Config;
use crate::db;

/// Resolved identity from either JWT or API token.
struct TurboAuth {
  user_id: Uuid,
  /// For API token auth, the org_id is known from the token.
  org_id: Option<Uuid>,
  /// For API token auth, the token_id for revocation.
  token_id: Option<Uuid>,
}

/// Resolve auth from the Authorization header: try JWT first, then API token.
async fn resolve_turbo_auth(
  auth_header: Option<&str>,
  config: &Config,
  pool: &PgPool,
) -> Result<TurboAuth, HttpResponse> {
  let token = auth_header
    .and_then(|s| s.strip_prefix("Bearer "))
    .ok_or_else(|| {
      HttpResponse::Unauthorized()
        .json(serde_json::json!({"error": "Missing Authorization header"}))
    })?;

  // Try JWT first
  if let Ok(claims) = validate_token(token, &config.jwt_secret) {
    return Ok(TurboAuth {
      user_id: claims.sub,
      org_id: None,
      token_id: None,
    });
  }

  // Try API token
  let token_hash = hash_token(token);
  if let Ok(Some(db_token)) = db::api_tokens::find_by_hash(pool, &token_hash).await {
    let is_valid = db_token
      .expires_at
      .map(|exp| exp > chrono::Utc::now())
      .unwrap_or(true);

    if is_valid {
      // Fire-and-forget: update last_used_at
      let pool_bg = pool.clone();
      let token_id = db_token.id;
      tokio::spawn(async move {
        let _ = db::api_tokens::update_last_used(&pool_bg, token_id).await;
      });

      return Ok(TurboAuth {
        user_id: db_token.created_by,
        org_id: Some(db_token.org_id),
        token_id: Some(db_token.id),
      });
    }
  }

  Err(HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid or expired token"})))
}

// ─── GET /v2/user ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct UserResponse {
  user: VercelUser,
}

#[derive(Serialize)]
struct VercelUser {
  id: String,
  username: String,
  email: String,
  name: Option<String>,
  #[serde(rename = "createdAt")]
  created_at: i64,
}

pub async fn get_user(
  req: actix_web::HttpRequest,
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
) -> HttpResponse {
  let auth_header = req
    .headers()
    .get("Authorization")
    .and_then(|v| v.to_str().ok());

  let turbo_auth = match resolve_turbo_auth(auth_header, &config, &pool).await {
    Ok(a) => a,
    Err(resp) => return resp,
  };

  match db::users::find_by_id(&pool, turbo_auth.user_id).await {
    Ok(Some(user)) => HttpResponse::Ok().json(UserResponse {
      user: VercelUser {
        id: user.id.to_string(),
        username: user.email.clone(),
        email: user.email,
        name: if user.name.is_empty() {
          None
        } else {
          Some(user.name)
        },
        created_at: user.created_at.timestamp_millis(),
      },
    }),
    _ => HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" })),
  }
}

// ─── GET /v2/teams ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TeamsQuery {
  #[allow(dead_code)]
  pub limit: Option<i64>,
}

#[derive(Serialize)]
struct TeamsResponse {
  teams: Vec<VercelTeam>,
}

#[derive(Serialize)]
struct VercelTeam {
  id: String,
  slug: String,
  name: String,
  #[serde(rename = "createdAt")]
  created_at: i64,
  created: String,
  membership: TeamMembership,
}

#[derive(Serialize)]
struct TeamMembership {
  role: String,
}

pub async fn list_teams(
  req: actix_web::HttpRequest,
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
  _query: Query<TeamsQuery>,
) -> HttpResponse {
  let auth_header = req
    .headers()
    .get("Authorization")
    .and_then(|v| v.to_str().ok());

  let turbo_auth = match resolve_turbo_auth(auth_header, &config, &pool).await {
    Ok(a) => a,
    Err(resp) => return resp,
  };

  // Return organizations as "teams" since that's the Vercel mental model
  match db::organizations::list_by_user(&pool, turbo_auth.user_id).await {
    Ok(orgs) => {
      let teams: Vec<VercelTeam> = orgs
        .into_iter()
        .map(|org| {
          let created = org.created_at.to_rfc3339();
          VercelTeam {
            id: format!("team_{}", org.id),
            slug: org.slug,
            name: org.name,
            created_at: org.created_at.timestamp_millis(),
            created,
            membership: TeamMembership {
              role: "OWNER".to_string(),
            },
          }
        })
        .collect();
      HttpResponse::Ok().json(TeamsResponse { teams })
    }
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to list teams: {e}")
    })),
  }
}

// ─── GET /v5/user/tokens/current ─────────────────────────────────────────────

#[derive(Serialize)]
struct TokenMetaResponse {
  token: TokenMeta,
}

#[derive(Serialize)]
struct TokenMeta {
  id: String,
  name: String,
  #[serde(rename = "type")]
  token_type: String,
  scopes: Vec<TokenScope>,
  #[serde(rename = "activeAt")]
  active_at: i64,
  #[serde(rename = "createdAt")]
  created_at: i64,
}

#[derive(Serialize)]
struct TokenScope {
  #[serde(rename = "type")]
  scope_type: String,
  #[serde(rename = "createdAt")]
  created_at: i64,
  #[serde(rename = "teamId")]
  team_id: Option<String>,
}

pub async fn get_current_token(
  req: actix_web::HttpRequest,
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
) -> HttpResponse {
  let auth_header = req
    .headers()
    .get("Authorization")
    .and_then(|v| v.to_str().ok());

  let turbo_auth = match resolve_turbo_auth(auth_header, &config, &pool).await {
    Ok(a) => a,
    Err(resp) => return resp,
  };

  let now = chrono::Utc::now().timestamp_millis();
  let (id, name) = match turbo_auth.token_id {
    Some(tid) => (tid.to_string(), "API Token".to_string()),
    None => (
      turbo_auth.user_id.to_string(),
      "Dashboard Token".to_string(),
    ),
  };

  HttpResponse::Ok().json(TokenMetaResponse {
    token: TokenMeta {
      id,
      name,
      token_type: "access".to_string(),
      scopes: vec![TokenScope {
        scope_type: "full_access".to_string(),
        created_at: now,
        team_id: turbo_auth.org_id.map(|id| format!("team_{}", id)),
      }],
      active_at: now,
      created_at: now,
    },
  })
}

// ─── DELETE /v3/user/tokens/current ──────────────────────────────────────────

pub async fn delete_current_token(
  req: actix_web::HttpRequest,
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
) -> HttpResponse {
  let auth_header = req
    .headers()
    .get("Authorization")
    .and_then(|v| v.to_str().ok());

  let turbo_auth = match resolve_turbo_auth(auth_header, &config, &pool).await {
    Ok(a) => a,
    Err(resp) => return resp,
  };

  // For API tokens, revoke the token; for JWT users, clear sessions
  if let Some(token_id) = turbo_auth.token_id {
    let _ = db::api_tokens::revoke(&pool, token_id).await;
  } else {
    let _ = db::sessions::delete_by_user(&pool, turbo_auth.user_id).await;
  }

  HttpResponse::Ok().json(serde_json::json!({}))
}
