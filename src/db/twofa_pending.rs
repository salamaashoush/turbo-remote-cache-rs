use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::twofa_pending::TwofaPending;

pub async fn create(
  pool: &PgPool,
  user_id: Uuid,
  token_hash: &str,
  method: &str,
  expires_at: DateTime<Utc>,
) -> Result<TwofaPending, sqlx::Error> {
  sqlx::query_as::<_, TwofaPending>(
    "INSERT INTO twofa_pending (user_id, token_hash, method, expires_at) VALUES ($1, $2, $3, $4) RETURNING *",
  )
  .bind(user_id)
  .bind(token_hash)
  .bind(method)
  .bind(expires_at)
  .fetch_one(pool)
  .await
}

pub async fn find_by_hash(
  pool: &PgPool,
  token_hash: &str,
) -> Result<Option<TwofaPending>, sqlx::Error> {
  sqlx::query_as::<_, TwofaPending>(
    "SELECT * FROM twofa_pending WHERE token_hash = $1 AND expires_at > now()",
  )
  .bind(token_hash)
  .fetch_optional(pool)
  .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM twofa_pending WHERE id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn delete_by_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM twofa_pending WHERE user_id = $1")
    .bind(user_id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
  let result = sqlx::query("DELETE FROM twofa_pending WHERE expires_at < now()")
    .execute(pool)
    .await?;
  Ok(result.rows_affected())
}
