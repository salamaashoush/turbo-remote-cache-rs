use std::sync::Arc;

use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest, dev::Payload};
use futures_util::future::LocalBoxFuture;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;

use super::jwt::{Claims, validate_token};

/// Extractor that validates JWT and returns Claims. Returns 401 if invalid.
pub struct AuthenticatedUser(pub Claims);

impl FromRequest for AuthenticatedUser {
  type Error = actix_web::Error;
  type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

  fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
    let config = req.app_data::<Data<Arc<Config>>>().cloned();
    let auth_header = req
      .headers()
      .get("Authorization")
      .and_then(|v| v.to_str().ok())
      .and_then(|s| s.strip_prefix("Bearer "))
      .map(|s| s.to_string());

    Box::pin(async move {
      let config = config
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Missing configuration"))?;

      let token = auth_header.ok_or_else(|| {
        actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header")
      })?;

      let claims = validate_token(&token, &config.jwt_secret)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid or expired token"))?;

      Ok(AuthenticatedUser(claims))
    })
  }
}

/// Extractor that validates JWT + org membership. Returns 403 if not a member.
pub struct OrgMember {
  pub claims: Claims,
  pub org_id: Uuid,
  pub org_role: String,
}

impl FromRequest for OrgMember {
  type Error = actix_web::Error;
  type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

  fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    let auth_fut = AuthenticatedUser::from_request(req, payload);
    let pool = req.app_data::<Data<PgPool>>().cloned();
    let path = req.match_info().get("org_id").map(|s| s.to_string());

    Box::pin(async move {
      let AuthenticatedUser(claims) = auth_fut.await?;

      let pool =
        pool.ok_or_else(|| actix_web::error::ErrorInternalServerError("Database not available"))?;

      let org_id_str =
        path.ok_or_else(|| actix_web::error::ErrorBadRequest("Missing org_id in path"))?;

      let org_id = Uuid::parse_str(&org_id_str)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid org_id format"))?;

      // Super admins have access to all orgs
      if claims.role == "super_admin" {
        return Ok(OrgMember {
          claims,
          org_id,
          org_role: "admin".to_string(),
        });
      }

      let member = crate::db::org_members::get_member_role(&pool, org_id, claims.sub)
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database error"))?
        .ok_or_else(|| actix_web::error::ErrorForbidden("Not a member of this organization"))?;

      Ok(OrgMember {
        claims,
        org_id,
        org_role: member.role,
      })
    })
  }
}

/// Extractor that validates JWT + org admin role. Returns 403 if not admin.
pub struct OrgAdmin {
  pub claims: Claims,
  pub org_id: Uuid,
}

impl FromRequest for OrgAdmin {
  type Error = actix_web::Error;
  type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

  fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    let member_fut = OrgMember::from_request(req, payload);

    Box::pin(async move {
      let member = member_fut.await?;

      if member.org_role != "admin" && member.org_role != "owner" {
        return Err(actix_web::error::ErrorForbidden("Admin access required"));
      }

      Ok(OrgAdmin {
        claims: member.claims,
        org_id: member.org_id,
      })
    })
  }
}

/// Extractor that validates JWT + super_admin role. Returns 403 if not super_admin.
pub struct SuperAdmin(pub Claims);

impl FromRequest for SuperAdmin {
  type Error = actix_web::Error;
  type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

  fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    let auth_fut = AuthenticatedUser::from_request(req, payload);

    Box::pin(async move {
      let AuthenticatedUser(claims) = auth_fut.await?;

      if claims.role != "super_admin" {
        return Err(actix_web::error::ErrorForbidden(
          "Super admin access required",
        ));
      }

      Ok(SuperAdmin(claims))
    })
  }
}
