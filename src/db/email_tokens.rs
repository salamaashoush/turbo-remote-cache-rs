use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::email_token::EmailToken;

pub async fn create(
  pool: &PgPool,
  user_id: Uuid,
  token_hash: &str,
  token_type: &str,
  expires_at: DateTime<Utc>,
) -> Result<EmailToken, sqlx::Error> {
  sqlx::query_as::<_, EmailToken>(
    "INSERT INTO email_tokens (user_id, token_hash, token_type, expires_at)
     VALUES ($1, $2, $3, $4) RETURNING *",
  )
  .bind(user_id)
  .bind(token_hash)
  .bind(token_type)
  .bind(expires_at)
  .fetch_one(pool)
  .await
}

pub async fn find_by_hash(
  pool: &PgPool,
  token_hash: &str,
  token_type: &str,
) -> Result<Option<EmailToken>, sqlx::Error> {
  sqlx::query_as::<_, EmailToken>(
    "SELECT * FROM email_tokens WHERE token_hash = $1 AND token_type = $2 AND expires_at > now()",
  )
  .bind(token_hash)
  .bind(token_type)
  .fetch_optional(pool)
  .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM email_tokens WHERE id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn delete_by_user(
  pool: &PgPool,
  user_id: Uuid,
  token_type: &str,
) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM email_tokens WHERE user_id = $1 AND token_type = $2")
    .bind(user_id)
    .bind(token_type)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
  let result = sqlx::query("DELETE FROM email_tokens WHERE expires_at < now()")
    .execute(pool)
    .await?;
  Ok(result.rows_affected())
}
