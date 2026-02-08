use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::api_token::ApiToken;

#[allow(clippy::too_many_arguments)]
pub async fn create(
  pool: &PgPool,
  org_id: Uuid,
  team_id: Option<Uuid>,
  name: &str,
  token_hash: &str,
  token_prefix: &str,
  scopes: &[String],
  created_by: Uuid,
  expires_at: Option<chrono::DateTime<Utc>>,
) -> Result<ApiToken, sqlx::Error> {
  sqlx::query_as::<_, ApiToken>(
    "INSERT INTO api_tokens (org_id, team_id, name, token_hash, token_prefix, scopes, created_by, expires_at)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
  )
  .bind(org_id)
  .bind(team_id)
  .bind(name)
  .bind(token_hash)
  .bind(token_prefix)
  .bind(scopes)
  .bind(created_by)
  .bind(expires_at)
  .fetch_one(pool)
  .await
}

pub async fn find_by_hash(
  pool: &PgPool,
  token_hash: &str,
) -> Result<Option<ApiToken>, sqlx::Error> {
  sqlx::query_as::<_, ApiToken>(
    "SELECT * FROM api_tokens WHERE token_hash = $1 AND revoked = false",
  )
  .bind(token_hash)
  .fetch_optional(pool)
  .await
}

pub async fn list_by_org(pool: &PgPool, org_id: Uuid) -> Result<Vec<ApiToken>, sqlx::Error> {
  sqlx::query_as::<_, ApiToken>(
    "SELECT * FROM api_tokens WHERE org_id = $1 ORDER BY created_at DESC",
  )
  .bind(org_id)
  .fetch_all(pool)
  .await
}

pub async fn revoke(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("UPDATE api_tokens SET revoked = true WHERE id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn update_last_used(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("UPDATE api_tokens SET last_used_at = now() WHERE id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn count_active(pool: &PgPool) -> Result<i64, sqlx::Error> {
  let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api_tokens WHERE revoked = false")
    .fetch_one(pool)
    .await?;
  Ok(row.0)
}

pub async fn revoke_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
  let result = sqlx::query(
    "UPDATE api_tokens SET revoked = true WHERE expires_at < now() AND revoked = false",
  )
  .execute(pool)
  .await?;
  Ok(result.rows_affected())
}

pub async fn count_by_org(pool: &PgPool, org_id: Uuid) -> Result<i64, sqlx::Error> {
  let row: (i64,) =
    sqlx::query_as("SELECT COUNT(*) FROM api_tokens WHERE org_id = $1 AND revoked = false")
      .bind(org_id)
      .fetch_one(pool)
      .await?;
  Ok(row.0)
}
