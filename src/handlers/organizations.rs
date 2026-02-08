use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Json, Path};
use sqlx::PgPool;
use tracing::warn;
use uuid::Uuid;

use crate::auth::extractors::{AuthenticatedUser, OrgAdmin, OrgMember};
use crate::config::Config;
use crate::db;
use crate::email::Mailer;
use crate::models::organization::{
  AddMemberRequest, UpdateMemberRoleRequest, UpdateOrgRequest,
};

pub async fn list_orgs(auth: AuthenticatedUser, pool: Data<PgPool>) -> HttpResponse {
  match db::organizations::list_by_user(&pool, auth.0.sub).await {
    Ok(orgs) => HttpResponse::Ok().json(orgs),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to list organizations: {e}")
    })),
  }
}

pub async fn get_org(member: OrgMember, pool: Data<PgPool>) -> HttpResponse {
  match db::organizations::find_by_id(&pool, member.org_id).await {
    Ok(Some(org)) => HttpResponse::Ok().json(org),
    Ok(None) => {
      HttpResponse::NotFound().json(serde_json::json!({ "error": "Organization not found" }))
    }
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Database error: {e}")
    })),
  }
}

pub async fn update_org(
  admin: OrgAdmin,
  pool: Data<PgPool>,
  body: Json<UpdateOrgRequest>,
) -> HttpResponse {
  match db::organizations::update(
    &pool,
    admin.org_id,
    body.name.as_deref(),
    body.slug.as_deref(),
  )
  .await
  {
    Ok(org) => HttpResponse::Ok().json(org),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to update organization: {e}")
    })),
  }
}

pub async fn delete_org(
  auth: AuthenticatedUser,
  pool: Data<PgPool>,
  path: Path<Uuid>,
) -> HttpResponse {
  // Only super_admin can delete orgs — regular users have a single org and cannot delete it
  if auth.0.role != "super_admin" {
    return HttpResponse::Forbidden().json(serde_json::json!({
      "error": "Only super admins can delete organizations"
    }));
  }

  let org_id = path.into_inner();

  if db::organizations::find_by_id(&pool, org_id)
    .await
    .ok()
    .flatten()
    .is_none()
  {
    return HttpResponse::NotFound().json(serde_json::json!({ "error": "Organization not found" }));
  }

  match db::organizations::delete(&pool, org_id).await {
    Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "message": "Organization deleted" })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to delete organization: {e}")
    })),
  }
}

pub async fn list_members(member: OrgMember, pool: Data<PgPool>) -> HttpResponse {
  match db::org_members::list_members(&pool, member.org_id).await {
    Ok(members) => HttpResponse::Ok().json(members),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to list members: {e}")
    })),
  }
}

pub async fn add_member(
  admin: OrgAdmin,
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
  mailer: Data<Option<Mailer>>,
  body: Json<AddMemberRequest>,
) -> HttpResponse {
  let role = body.role.as_deref().unwrap_or("member");

  const VALID_ROLES: &[&str] = &["member", "admin"];
  if !VALID_ROLES.contains(&role) {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": format!("Invalid role '{}'. Valid roles: member, admin", role)
    }));
  }

  // Find user by email
  let user = match db::users::find_by_email(&pool, &body.email).await {
    Ok(Some(u)) => u,
    Ok(None) => {
      return HttpResponse::NotFound().json(serde_json::json!({
        "error": "User not found with that email"
      }));
    }
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Database error: {e}")
      }));
    }
  };

  let member = match db::org_members::add_member(&pool, admin.org_id, user.id, role).await {
    Ok(m) => m,
    Err(e) => {
      return HttpResponse::Conflict().json(serde_json::json!({
        "error": format!("Failed to add member: {e}")
      }));
    }
  };

  // Send invitation email
  if let Some(mailer) = mailer.as_ref() {
    let mailer = mailer.clone();
    let app_url = config.app_url.clone();
    let inviter_name = admin.claims.email.clone();
    let user_email = user.email.clone();

    // Look up org name for the email
    let org_name = match db::organizations::find_by_id(&pool, admin.org_id).await {
      Ok(Some(org)) => org.name,
      _ => "an organization".to_string(),
    };

    tokio::spawn(async move {
      let (subj, html) =
        crate::email::templates::org_invitation(&inviter_name, &org_name, &app_url);
      if let Err(e) = mailer.send(&user_email, &subj, &html).await {
        warn!("Failed to send invitation email to {}: {}", user_email, e);
      }
    });
  }

  HttpResponse::Created().json(member)
}

pub async fn update_member_role(
  admin: OrgAdmin,
  pool: Data<PgPool>,
  path: Path<(Uuid, Uuid)>,
  body: Json<UpdateMemberRoleRequest>,
) -> HttpResponse {
  let (_org_id, user_id) = path.into_inner();

  const VALID_ROLES: &[&str] = &["member", "admin"];
  if !VALID_ROLES.contains(&body.role.as_str()) {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": format!("Invalid role '{}'. Valid roles: member, admin", body.role)
    }));
  }

  match db::org_members::update_role(&pool, admin.org_id, user_id, &body.role).await {
    Ok(member) => HttpResponse::Ok().json(member),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to update member role: {e}")
    })),
  }
}

pub async fn remove_member(
  admin: OrgAdmin,
  pool: Data<PgPool>,
  path: Path<(Uuid, Uuid)>,
) -> HttpResponse {
  let (_org_id, user_id) = path.into_inner();

  // Can't remove yourself
  if user_id == admin.claims.sub {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": "Cannot remove yourself from the organization"
    }));
  }

  match db::org_members::remove_member(&pool, admin.org_id, user_id).await {
    Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "message": "Member removed" })),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to remove member: {e}")
    })),
  }
}
