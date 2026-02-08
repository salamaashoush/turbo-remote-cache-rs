use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::session::Session;

pub async fn create(
  pool: &PgPool,
  user_id: Uuid,
  token_hash: &str,
  expires_at: DateTime<Utc>,
) -> Result<Session, sqlx::Error> {
  sqlx::query_as::<_, Session>(
    "INSERT INTO sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3) RETURNING *",
  )
  .bind(user_id)
  .bind(token_hash)
  .bind(expires_at)
  .fetch_one(pool)
  .await
}

pub async fn find_by_hash(pool: &PgPool, token_hash: &str) -> Result<Option<Session>, sqlx::Error> {
  sqlx::query_as::<_, Session>(
    "SELECT * FROM sessions WHERE token_hash = $1 AND expires_at > now()",
  )
  .bind(token_hash)
  .fetch_optional(pool)
  .await
}

pub async fn delete_by_hash(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
    .bind(token_hash)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn delete_by_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM sessions WHERE user_id = $1")
    .bind(user_id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn count_active(pool: &PgPool) -> Result<i64, sqlx::Error> {
  let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE expires_at > now()")
    .fetch_one(pool)
    .await?;
  Ok(row.0)
}

pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
  let result = sqlx::query("DELETE FROM sessions WHERE expires_at < now()")
    .execute(pool)
    .await?;
  Ok(result.rows_affected())
}
