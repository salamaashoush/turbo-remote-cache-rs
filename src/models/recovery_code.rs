use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct RecoveryCode {
  pub id: Uuid,
  pub user_id: Uuid,
  pub code_hash: String,
  pub used: bool,
  pub created_at: DateTime<Utc>,
}
