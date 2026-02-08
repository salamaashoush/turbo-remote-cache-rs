use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Json, Path, Query};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::extractors::SuperAdmin;
use crate::config::Config;
use crate::db;
use crate::email::Mailer;
use crate::models::cache_event::{AnalyticsOverview, PlatformStats};
use crate::models::organization::{
  AdminCreateOrgRequest, AdminOrgDetailResponse, AdminOrgMemberEntry, AdminOrgOwner,
  AdminOrgTeamEntry, TransferOwnershipRequest, UpdateOrgLimitsRequest,
};
use crate::models::user::{
  AdminAddToOrgRequest, AdminUserDetailResponse, AdminUserOrgEntry, AdminUserResponse, UserResponse,
};
use crate::storage::StorageStore;

#[derive(serde::Deserialize)]
pub struct PaginationQuery {
  pub limit: Option<i64>,
  pub offset: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct AnalyticsQuery {
  pub period: Option<String>,
  pub granularity: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateUserRequest {
  pub role: Option<String>,
}

fn parse_period(period: &str) -> Duration {
  match period {
    "1d" => Duration::days(1),
    "7d" => Duration::days(7),
    "30d" => Duration::days(30),
    "90d" => Duration::days(90),
    _ => Duration::days(30),
  }
}

// ─── Existing handlers ───

pub async fn list_orgs(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  query: Query<PaginationQuery>,
) -> HttpResponse {
  let limit = query.limit.unwrap_or(50).min(100);
  let offset = query.offset.unwrap_or(0);

  match db::organizations::list_all_with_counts(&pool, limit, offset).await {
    Ok(orgs) => HttpResponse::Ok().json(orgs),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to list organizations: {e}")
    })),
  }
}

pub async fn list_users(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  query: Query<PaginationQuery>,
) -> HttpResponse {
  let limit = query.limit.unwrap_or(50).min(100);
  let offset = query.offset.unwrap_or(0);

  match db::users::list_all_with_details(&pool, limit, offset).await {
    Ok(users) => {
      let responses: Vec<AdminUserResponse> =
        users.into_iter().map(AdminUserResponse::from).collect();
      HttpResponse::Ok().json(responses)
    }
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to list users: {e}")
    })),
  }
}

pub async fn platform_stats(_admin: SuperAdmin, pool: Data<PgPool>) -> HttpResponse {
  let total_users = db::users::count_all(&pool).await.unwrap_or(0);
  let total_orgs = db::organizations::count_all(&pool).await.unwrap_or(0);
  let total_events = db::cache_events::get_total_events(&pool).await.unwrap_or(0);
  let total_bytes = db::cache_events::get_total_bytes(&pool).await.unwrap_or(0);
  let total_active_tokens = db::api_tokens::count_active(&pool).await.unwrap_or(0);
  let total_active_sessions = db::sessions::count_active(&pool).await.unwrap_or(0);

  let since = Utc::now() - Duration::days(365 * 10);
  let counts = db::cache_events::get_platform_overview(&pool, since).await;
  let (total_hits, total_misses, total_puts, total_duration_ms) = match counts {
    Ok(c) => (c.hits, c.misses, c.puts, c.total_duration_ms),
    Err(_) => (0, 0, 0, 0),
  };

  HttpResponse::Ok().json(PlatformStats {
    total_users,
    total_orgs,
    total_events,
    total_bytes,
    total_active_tokens,
    total_active_sessions,
    total_hits,
    total_misses,
    total_puts,
    total_duration_ms,
  })
}

pub async fn platform_overview(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  query: Query<AnalyticsQuery>,
) -> HttpResponse {
  let period = query.period.as_deref().unwrap_or("30d");
  let since = Utc::now() - parse_period(period);

  match db::cache_events::get_platform_overview(&pool, since).await {
    Ok(counts) => {
      let hit_rate = if counts.total > 0 {
        counts.hits as f64 / counts.total as f64
      } else {
        0.0
      };

      HttpResponse::Ok().json(AnalyticsOverview {
        period: period.to_string(),
        total_events: counts.total,
        hits: counts.hits,
        misses: counts.misses,
        puts: counts.puts,
        hit_rate,
        total_bytes_saved: counts.total_bytes,
        estimated_time_saved_ms: counts.total_duration_ms,
      })
    }
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to get platform analytics: {e}")
    })),
  }
}

pub async fn platform_timeline(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  query: Query<AnalyticsQuery>,
) -> HttpResponse {
  let period = query.period.as_deref().unwrap_or("30d");
  let granularity = query.granularity.as_deref().unwrap_or("day");
  let since = Utc::now() - parse_period(period);

  match db::cache_events::get_platform_timeline(&pool, since, granularity).await {
    Ok(points) => HttpResponse::Ok().json(points),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to get platform timeline: {e}")
    })),
  }
}

pub async fn platform_org_breakdown(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  query: Query<AnalyticsQuery>,
) -> HttpResponse {
  let period = query.period.as_deref().unwrap_or("30d");
  let since = Utc::now() - parse_period(period);

  match db::cache_events::get_org_breakdown(&pool, since).await {
    Ok(breakdown) => HttpResponse::Ok().json(breakdown),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to get org breakdown: {e}")
    })),
  }
}

pub async fn update_user(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  path: Path<Uuid>,
  body: Json<UpdateUserRequest>,
) -> HttpResponse {
  let user_id = path.into_inner();

  if let Some(role) = &body.role {
    const VALID_ROLES: &[&str] = &["user", "super_admin"];
    if !VALID_ROLES.contains(&role.as_str()) {
      return HttpResponse::BadRequest().json(serde_json::json!({
        "error": format!("Invalid role '{}'. Valid roles: user, super_admin", role)
      }));
    }

    match db::users::update_role(&pool, user_id, role).await {
      Ok(user) => HttpResponse::Ok().json(UserResponse::from(user)),
      Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Failed to update user: {e}")
      })),
    }
  } else {
    HttpResponse::BadRequest().json(serde_json::json!({
      "error": "No fields to update"
    }))
  }
}

pub async fn delete_org(_admin: SuperAdmin, pool: Data<PgPool>, path: Path<Uuid>) -> HttpResponse {
  let org_id = path.into_inner();

  match db::organizations::delete(&pool, org_id).await {
    Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "message": "Organization deleted" })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to delete organization: {e}")
    })),
  }
}

pub async fn email_status(_admin: SuperAdmin, config: Data<Arc<Config>>) -> HttpResponse {
  let (configured, host) = match &config.smtp {
    Some(smtp) => (true, Some(smtp.host.clone())),
    None => (false, None),
  };
  HttpResponse::Ok().json(serde_json::json!({
    "configured": configured,
    "host": host,
  }))
}

// ─── New admin handlers ───

pub async fn get_user(_admin: SuperAdmin, pool: Data<PgPool>, path: Path<Uuid>) -> HttpResponse {
  let user_id = path.into_inner();

  let user = match db::users::find_by_id(&pool, user_id).await {
    Ok(Some(u)) => u,
    Ok(None) => {
      return HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" }));
    }
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Database error: {e}")
      }));
    }
  };

  // Get user's orgs with roles
  let orgs = match sqlx::query_as::<_, (Uuid, String, String, String)>(
    "SELECT o.id, o.name, o.slug, om.role
     FROM org_members om
     JOIN organizations o ON o.id = om.org_id
     WHERE om.user_id = $1
     ORDER BY o.name",
  )
  .bind(user_id)
  .fetch_all(pool.get_ref())
  .await
  {
    Ok(rows) => rows
      .into_iter()
      .map(|(id, name, slug, role)| AdminUserOrgEntry {
        id,
        name,
        slug,
        role,
      })
      .collect::<Vec<_>>(),
    Err(_) => vec![],
  };

  let session_count = db::users::count_sessions(&pool, user_id).await.unwrap_or(0);
  let token_count = db::users::count_tokens(&pool, user_id).await.unwrap_or(0);

  // Get last_active from sessions
  let last_active: Option<chrono::DateTime<Utc>> =
    sqlx::query_as::<_, (Option<chrono::DateTime<Utc>>,)>(
      "SELECT MAX(created_at) FROM sessions WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map(|r| r.0)
    .unwrap_or(None);

  let twofa_enabled = user.twofa_method != "none";

  HttpResponse::Ok().json(AdminUserDetailResponse {
    id: user.id,
    email: user.email,
    name: user.name,
    role: user.role,
    is_active: user.is_active,
    email_verified: user.email_verified,
    twofa_method: user.twofa_method,
    twofa_enabled,
    created_at: user.created_at,
    last_active,
    orgs,
    session_count,
    token_count,
  })
}

pub async fn activate_user(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  path: Path<Uuid>,
) -> HttpResponse {
  let user_id = path.into_inner();

  match db::users::set_active(&pool, user_id, true).await {
    Ok(user) => HttpResponse::Ok().json(UserResponse::from(user)),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to activate user: {e}")
    })),
  }
}

pub async fn deactivate_user(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  path: Path<Uuid>,
) -> HttpResponse {
  let user_id = path.into_inner();

  match db::users::set_active(&pool, user_id, false).await {
    Ok(_) => {
      // Kill all sessions
      let _ = db::sessions::delete_by_user(&pool, user_id).await;
      HttpResponse::Ok().json(serde_json::json!({ "message": "User deactivated" }))
    }
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to deactivate user: {e}")
    })),
  }
}

pub async fn delete_user_handler(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  path: Path<Uuid>,
) -> HttpResponse {
  let user_id = path.into_inner();

  match db::users::delete_user(&pool, user_id).await {
    Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "message": "User deleted" })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to delete user: {e}")
    })),
  }
}

pub async fn force_logout(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  path: Path<Uuid>,
) -> HttpResponse {
  let user_id = path.into_inner();

  match db::sessions::delete_by_user(&pool, user_id).await {
    Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "message": "All sessions terminated" })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to force logout: {e}")
    })),
  }
}

pub async fn admin_password_reset(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
  mailer: Data<Option<Mailer>>,
  path: Path<Uuid>,
) -> HttpResponse {
  let user_id = path.into_inner();

  let mailer = match mailer.as_ref() {
    Some(m) => m.clone(),
    None => {
      return HttpResponse::BadRequest().json(serde_json::json!({
        "error": "SMTP is not configured"
      }));
    }
  };

  let user = match db::users::find_by_id(&pool, user_id).await {
    Ok(Some(u)) => u,
    Ok(None) => {
      return HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" }));
    }
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Database error: {e}")
      }));
    }
  };

  // Delete any existing reset tokens for this user
  let _ = db::email_tokens::delete_by_user(&pool, user.id, "reset_password").await;

  let raw_token = crate::auth::api_token::generate_token();
  let token_hash = crate::auth::api_token::hash_token(&raw_token);
  let expires = Utc::now() + Duration::hours(1);

  if let Err(e) =
    db::email_tokens::create(&pool, user.id, &token_hash, "reset_password", expires).await
  {
    return HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to create reset token: {e}")
    }));
  }

  let reset_url = format!("{}/reset-password?token={}", config.app_url, raw_token);
  let (subj, html) = crate::email::templates::password_reset(&user.name, &reset_url);

  let email = user.email.clone();
  tokio::spawn(async move {
    if let Err(e) = mailer.send(&email, &subj, &html).await {
      tracing::warn!(
        "Failed to send admin-initiated password reset to {}: {}",
        email,
        e
      );
    }
  });

  HttpResponse::Ok().json(serde_json::json!({ "message": "Password reset email sent" }))
}

pub async fn add_user_to_org(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  path: Path<Uuid>,
  body: Json<AdminAddToOrgRequest>,
) -> HttpResponse {
  let user_id = path.into_inner();
  let role = body.role.as_deref().unwrap_or("member");

  // Validate role
  const VALID_ROLES: &[&str] = &["admin", "member"];
  if !VALID_ROLES.contains(&role) {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": format!("Invalid role '{}'. Valid roles: admin, member", role)
    }));
  }

  // Check user exists
  if let Ok(None) | Err(_) = db::users::find_by_id(&pool, user_id).await {
    return HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" }));
  }

  // Check org exists
  if let Ok(None) | Err(_) = db::organizations::find_by_id(&pool, body.org_id).await {
    return HttpResponse::NotFound().json(serde_json::json!({ "error": "Organization not found" }));
  }

  match db::org_members::add_member(&pool, body.org_id, user_id, role).await {
    Ok(member) => HttpResponse::Ok().json(member),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to add user to org: {e}")
    })),
  }
}

pub async fn remove_user_from_org(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  path: Path<(Uuid, Uuid)>,
) -> HttpResponse {
  let (user_id, org_id) = path.into_inner();

  match db::org_members::remove_member(&pool, org_id, user_id).await {
    Ok(()) => {
      HttpResponse::Ok().json(serde_json::json!({ "message": "User removed from organization" }))
    }
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to remove user from org: {e}")
    })),
  }
}

pub async fn get_org(_admin: SuperAdmin, pool: Data<PgPool>, path: Path<Uuid>) -> HttpResponse {
  let org_id = path.into_inner();

  let org = match db::organizations::find_by_id(&pool, org_id).await {
    Ok(Some(o)) => o,
    Ok(None) => {
      return HttpResponse::NotFound()
        .json(serde_json::json!({ "error": "Organization not found" }));
    }
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Database error: {e}")
      }));
    }
  };

  // Get owner info
  let owner = match db::users::find_by_id(&pool, org.owner_id).await {
    Ok(Some(u)) => AdminOrgOwner {
      id: u.id,
      email: u.email,
      name: u.name,
    },
    _ => AdminOrgOwner {
      id: org.owner_id,
      email: "unknown".to_string(),
      name: "Unknown".to_string(),
    },
  };

  // Get members
  let members = match db::org_members::list_members(&pool, org_id).await {
    Ok(ms) => ms
      .into_iter()
      .map(|m| AdminOrgMemberEntry {
        id: m.user_id,
        email: m.email,
        name: m.name,
        role: m.role,
      })
      .collect::<Vec<_>>(),
    Err(_) => vec![],
  };

  // Get teams with member counts
  let teams = match sqlx::query_as::<_, (Uuid, String, i64)>(
    "SELECT t.id, t.name, (SELECT COUNT(*) FROM team_members tm WHERE tm.team_id = t.id) as member_count
     FROM teams t
     WHERE t.org_id = $1
     ORDER BY t.name",
  )
  .bind(org_id)
  .fetch_all(pool.get_ref())
  .await
  {
    Ok(rows) => rows
      .into_iter()
      .map(|(id, name, member_count)| AdminOrgTeamEntry {
        id,
        name,
        member_count,
      })
      .collect::<Vec<_>>(),
    Err(_) => vec![],
  };

  let token_count = db::api_tokens::count_by_org(&pool, org_id)
    .await
    .unwrap_or(0);
  let total_cache_bytes = db::cache_events::total_bytes_by_org(&pool, org_id)
    .await
    .unwrap_or(0);
  let total_cache_events = db::cache_events::total_events_by_org(&pool, org_id)
    .await
    .unwrap_or(0);

  HttpResponse::Ok().json(AdminOrgDetailResponse {
    id: org.id,
    name: org.name,
    slug: org.slug,
    owner,
    members,
    teams,
    token_count,
    cache_size_limit_bytes: org.cache_size_limit_bytes,
    max_tokens: org.max_tokens,
    total_cache_bytes,
    total_cache_events,
    created_at: org.created_at,
  })
}

pub async fn create_org(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  body: Json<AdminCreateOrgRequest>,
) -> HttpResponse {
  if body.name.is_empty() || body.slug.is_empty() {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": "Name and slug are required"
    }));
  }

  // Find owner by email
  let owner = match db::users::find_by_email(&pool, &body.owner_email).await {
    Ok(Some(u)) => u,
    Ok(None) => {
      return HttpResponse::NotFound().json(serde_json::json!({
        "error": format!("User with email '{}' not found", body.owner_email)
      }));
    }
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Database error: {e}")
      }));
    }
  };

  // Check slug uniqueness
  if let Ok(Some(_)) = db::organizations::find_by_slug(&pool, &body.slug).await {
    return HttpResponse::Conflict().json(serde_json::json!({
      "error": "Organization slug already taken"
    }));
  }

  let org = match db::organizations::create(&pool, &body.name, &body.slug, owner.id).await {
    Ok(o) => o,
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Failed to create organization: {e}")
      }));
    }
  };

  // Add owner as admin member
  let _ = db::org_members::add_member(&pool, org.id, owner.id, "admin").await;

  // Create default team
  if let Ok(team) = db::teams::create(&pool, org.id, "Default", "default").await {
    let _ = db::team_members::add_member(&pool, team.id, owner.id, "admin").await;
  }

  HttpResponse::Created().json(org)
}

pub async fn update_org_limits(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  path: Path<Uuid>,
  body: Json<UpdateOrgLimitsRequest>,
) -> HttpResponse {
  let org_id = path.into_inner();

  match db::organizations::update_limits(
    &pool,
    org_id,
    body.cache_size_limit_bytes,
    body.max_tokens,
  )
  .await
  {
    Ok(org) => HttpResponse::Ok().json(org),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to update limits: {e}")
    })),
  }
}

pub async fn transfer_org_ownership(
  _admin: SuperAdmin,
  pool: Data<PgPool>,
  path: Path<Uuid>,
  body: Json<TransferOwnershipRequest>,
) -> HttpResponse {
  let org_id = path.into_inner();

  // Check new owner exists
  let new_owner = match db::users::find_by_id(&pool, body.new_owner_id).await {
    Ok(Some(u)) => u,
    Ok(None) => {
      return HttpResponse::NotFound().json(serde_json::json!({ "error": "New owner not found" }));
    }
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Database error: {e}")
      }));
    }
  };

  // Ensure new owner is a member (add as admin if not)
  if let Ok(None) = db::org_members::get_member_role(&pool, org_id, new_owner.id).await {
    let _ = db::org_members::add_member(&pool, org_id, new_owner.id, "admin").await;
  }

  match db::organizations::transfer_ownership(&pool, org_id, new_owner.id).await {
    Ok(org) => HttpResponse::Ok().json(org),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to transfer ownership: {e}")
    })),
  }
}

pub async fn purge_org_cache(
  _admin: SuperAdmin,
  storage: Data<StorageStore>,
  path: Path<Uuid>,
) -> HttpResponse {
  let org_id = path.into_inner();
  let prefix = format!("{}/", org_id);

  match storage.delete_prefix(&prefix).await {
    Ok(count) => HttpResponse::Ok().json(serde_json::json!({
      "message": format!("Purged {} artifacts", count),
      "count": count
    })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to purge cache: {e}")
    })),
  }
}
