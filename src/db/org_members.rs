use sqlx::PgPool;
use uuid::Uuid;

use crate::models::organization::{OrgMember, OrgMemberWithEmail};

pub async fn add_member(
  pool: &PgPool,
  org_id: Uuid,
  user_id: Uuid,
  role: &str,
) -> Result<OrgMember, sqlx::Error> {
  sqlx::query_as::<_, OrgMember>(
    "INSERT INTO org_members (org_id, user_id, role, joined_at) VALUES ($1, $2, $3, now()) RETURNING *",
  )
  .bind(org_id)
  .bind(user_id)
  .bind(role)
  .fetch_one(pool)
  .await
}

pub async fn get_member_role(
  pool: &PgPool,
  org_id: Uuid,
  user_id: Uuid,
) -> Result<Option<OrgMember>, sqlx::Error> {
  sqlx::query_as::<_, OrgMember>("SELECT * FROM org_members WHERE org_id = $1 AND user_id = $2")
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_members(
  pool: &PgPool,
  org_id: Uuid,
) -> Result<Vec<OrgMemberWithEmail>, sqlx::Error> {
  sqlx::query_as::<_, OrgMemberWithEmail>(
    "SELECT om.org_id, om.user_id, om.role, u.email, u.name, om.invited_at, om.joined_at
     FROM org_members om
     INNER JOIN users u ON om.user_id = u.id
     WHERE om.org_id = $1
     ORDER BY om.invited_at",
  )
  .bind(org_id)
  .fetch_all(pool)
  .await
}

pub async fn update_role(
  pool: &PgPool,
  org_id: Uuid,
  user_id: Uuid,
  role: &str,
) -> Result<OrgMember, sqlx::Error> {
  sqlx::query_as::<_, OrgMember>(
    "UPDATE org_members SET role = $1 WHERE org_id = $2 AND user_id = $3 RETURNING *",
  )
  .bind(role)
  .bind(org_id)
  .bind(user_id)
  .fetch_one(pool)
  .await
}

pub async fn remove_member(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM org_members WHERE org_id = $1 AND user_id = $2")
    .bind(org_id)
    .bind(user_id)
    .execute(pool)
    .await?;
  Ok(())
}
