use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ApiToken {
  pub id: Uuid,
  pub org_id: Uuid,
  pub team_id: Option<Uuid>,
  pub name: String,
  #[serde(skip_serializing)]
  pub token_hash: String,
  pub token_prefix: String,
  pub scopes: Vec<String>,
  pub created_by: Uuid,
  pub expires_at: Option<DateTime<Utc>>,
  pub last_used_at: Option<DateTime<Utc>>,
  pub revoked: bool,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
  pub name: String,
  pub team_id: Option<Uuid>,
  pub scopes: Option<Vec<String>>,
  pub expires_in_days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateTokenResponse {
  pub id: Uuid,
  pub name: String,
  pub token: String,
  pub token_prefix: String,
  pub scopes: Vec<String>,
  pub expires_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
}
