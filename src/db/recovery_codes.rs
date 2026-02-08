use sqlx::PgPool;
use uuid::Uuid;

use crate::models::recovery_code::RecoveryCode;

pub async fn create_batch(
  pool: &PgPool,
  user_id: Uuid,
  hashes: &[String],
) -> Result<(), sqlx::Error> {
  for hash in hashes {
    sqlx::query("INSERT INTO recovery_codes (user_id, code_hash) VALUES ($1, $2)")
      .bind(user_id)
      .bind(hash)
      .execute(pool)
      .await?;
  }
  Ok(())
}

pub async fn find_unused_by_hash(
  pool: &PgPool,
  user_id: Uuid,
  code_hash: &str,
) -> Result<Option<RecoveryCode>, sqlx::Error> {
  sqlx::query_as::<_, RecoveryCode>(
    "SELECT * FROM recovery_codes WHERE user_id = $1 AND code_hash = $2 AND used = false",
  )
  .bind(user_id)
  .bind(code_hash)
  .fetch_optional(pool)
  .await
}

pub async fn mark_used(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("UPDATE recovery_codes SET used = true WHERE id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn count_unused(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
  let row: (i64,) =
    sqlx::query_as("SELECT COUNT(*) FROM recovery_codes WHERE user_id = $1 AND used = false")
      .bind(user_id)
      .fetch_one(pool)
      .await?;
  Ok(row.0)
}

pub async fn delete_by_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM recovery_codes WHERE user_id = $1")
    .bind(user_id)
    .execute(pool)
    .await?;
  Ok(())
}
