use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Organization {
  pub id: Uuid,
  pub name: String,
  pub slug: String,
  pub owner_id: Uuid,
  pub storage_provider: Option<String>,
  pub storage_config: Option<serde_json::Value>,
  pub cache_size_limit_bytes: Option<i64>,
  pub max_tokens: Option<i32>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct OrgWithCounts {
  pub id: Uuid,
  pub name: String,
  pub slug: String,
  pub owner_id: Uuid,
  pub storage_provider: Option<String>,
  pub storage_config: Option<serde_json::Value>,
  pub cache_size_limit_bytes: Option<i64>,
  pub max_tokens: Option<i32>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub owner_email: String,
  pub member_count: i64,
  pub team_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
  pub name: Option<String>,
  pub slug: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct OrgMember {
  pub org_id: Uuid,
  pub user_id: Uuid,
  pub role: String,
  pub invited_at: DateTime<Utc>,
  pub joined_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct OrgMemberWithEmail {
  pub org_id: Uuid,
  pub user_id: Uuid,
  pub role: String,
  pub email: String,
  pub name: String,
  pub invited_at: DateTime<Utc>,
  pub joined_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
  pub email: String,
  pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
  pub role: String,
}

#[derive(Debug, Serialize)]
pub struct AdminOrgOwner {
  pub id: Uuid,
  pub email: String,
  pub name: String,
}

#[derive(Debug, Serialize)]
pub struct AdminOrgMemberEntry {
  pub id: Uuid,
  pub email: String,
  pub name: String,
  pub role: String,
}

#[derive(Debug, Serialize)]
pub struct AdminOrgTeamEntry {
  pub id: Uuid,
  pub name: String,
  pub member_count: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminOrgDetailResponse {
  pub id: Uuid,
  pub name: String,
  pub slug: String,
  pub owner: AdminOrgOwner,
  pub members: Vec<AdminOrgMemberEntry>,
  pub teams: Vec<AdminOrgTeamEntry>,
  pub token_count: i64,
  pub cache_size_limit_bytes: Option<i64>,
  pub max_tokens: Option<i32>,
  pub total_cache_bytes: i64,
  pub total_cache_events: i64,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgLimitsRequest {
  pub cache_size_limit_bytes: Option<i64>,
  pub max_tokens: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TransferOwnershipRequest {
  pub new_owner_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AdminCreateOrgRequest {
  pub name: String,
  pub slug: String,
  pub owner_email: String,
}
