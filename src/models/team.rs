use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Team {
  pub id: Uuid,
  pub org_id: Uuid,
  pub name: String,
  pub slug: String,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
  pub name: String,
  pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamRequest {
  pub name: Option<String>,
  pub slug: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TeamMember {
  pub team_id: Uuid,
  pub user_id: Uuid,
  pub role: String,
  pub added_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct TeamMemberWithEmail {
  pub team_id: Uuid,
  pub user_id: Uuid,
  pub role: String,
  pub email: String,
  pub name: String,
  pub added_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddTeamMemberRequest {
  pub user_id: Uuid,
  pub role: Option<String>,
}
