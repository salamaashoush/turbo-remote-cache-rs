use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;

pub async fn create_user(
  pool: &PgPool,
  email: &str,
  password_hash: &str,
  name: &str,
  role: &str,
) -> Result<User, sqlx::Error> {
  sqlx::query_as::<_, User>(
    "INSERT INTO users (email, password_hash, name, role) VALUES ($1, $2, $3, $4) RETURNING *",
  )
  .bind(email)
  .bind(password_hash)
  .bind(name)
  .bind(role)
  .fetch_one(pool)
  .await
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
  sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
  sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn update_user(
  pool: &PgPool,
  id: Uuid,
  name: Option<&str>,
  password_hash: Option<&str>,
) -> Result<User, sqlx::Error> {
  let user = find_by_id(pool, id)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;

  let name = name.unwrap_or(&user.name);
  let password_hash = password_hash.unwrap_or(&user.password_hash);

  sqlx::query_as::<_, User>(
    "UPDATE users SET name = $1, password_hash = $2 WHERE id = $3 RETURNING *",
  )
  .bind(name)
  .bind(password_hash)
  .bind(id)
  .fetch_one(pool)
  .await
}

pub async fn update_role(pool: &PgPool, id: Uuid, role: &str) -> Result<User, sqlx::Error> {
  sqlx::query_as::<_, User>("UPDATE users SET role = $1 WHERE id = $2 RETURNING *")
    .bind(role)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn list_all(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<User>, sqlx::Error> {
  sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2")
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn list_all_with_details(
  pool: &PgPool,
  limit: i64,
  offset: i64,
) -> Result<Vec<crate::models::user::UserWithDetails>, sqlx::Error> {
  sqlx::query_as::<_, crate::models::user::UserWithDetails>(
    "SELECT u.id, u.email, u.name, u.role, u.email_verified, u.is_active, u.created_at, u.updated_at,
            (SELECT o.name FROM org_members om JOIN organizations o ON o.id = om.org_id WHERE om.user_id = u.id LIMIT 1) as org_name,
            (SELECT MAX(s.created_at) FROM sessions s WHERE s.user_id = u.id) as last_active
     FROM users u
     ORDER BY u.created_at DESC
     LIMIT $1 OFFSET $2",
  )
  .bind(limit)
  .bind(offset)
  .fetch_all(pool)
  .await
}

pub async fn set_email_verified(pool: &PgPool, id: Uuid) -> Result<User, sqlx::Error> {
  sqlx::query_as::<_, User>("UPDATE users SET email_verified = true WHERE id = $1 RETURNING *")
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn update_password(
  pool: &PgPool,
  id: Uuid,
  password_hash: &str,
) -> Result<User, sqlx::Error> {
  sqlx::query_as::<_, User>("UPDATE users SET password_hash = $1 WHERE id = $2 RETURNING *")
    .bind(password_hash)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn count_all(pool: &PgPool) -> Result<i64, sqlx::Error> {
  let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
    .fetch_one(pool)
    .await?;
  Ok(row.0)
}

pub async fn set_totp_secret(pool: &PgPool, id: Uuid, secret: &str) -> Result<User, sqlx::Error> {
  sqlx::query_as::<_, User>("UPDATE users SET totp_secret = $1 WHERE id = $2 RETURNING *")
    .bind(secret)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn enable_totp(pool: &PgPool, id: Uuid) -> Result<User, sqlx::Error> {
  sqlx::query_as::<_, User>(
    "UPDATE users SET totp_enabled = true, twofa_method = 'totp' WHERE id = $1 RETURNING *",
  )
  .bind(id)
  .fetch_one(pool)
  .await
}

pub async fn enable_email_twofa(pool: &PgPool, id: Uuid) -> Result<User, sqlx::Error> {
  sqlx::query_as::<_, User>("UPDATE users SET twofa_method = 'email' WHERE id = $1 RETURNING *")
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn disable_twofa(pool: &PgPool, id: Uuid) -> Result<User, sqlx::Error> {
  sqlx::query_as::<_, User>(
    "UPDATE users SET twofa_method = 'none', totp_secret = NULL, totp_enabled = false WHERE id = $1 RETURNING *",
  )
  .bind(id)
  .fetch_one(pool)
  .await
}

pub async fn set_active(pool: &PgPool, id: Uuid, is_active: bool) -> Result<User, sqlx::Error> {
  sqlx::query_as::<_, User>("UPDATE users SET is_active = $1 WHERE id = $2 RETURNING *")
    .bind(is_active)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_user(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
  // Delete related data first (sessions, org_members, team_members, tokens, etc.)
  sqlx::query("DELETE FROM sessions WHERE user_id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  sqlx::query("DELETE FROM team_members WHERE user_id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  sqlx::query("DELETE FROM org_members WHERE user_id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  sqlx::query("DELETE FROM api_tokens WHERE created_by = $1")
    .bind(id)
    .execute(pool)
    .await?;
  sqlx::query("DELETE FROM email_tokens WHERE user_id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  sqlx::query("DELETE FROM twofa_pending WHERE user_id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  sqlx::query("DELETE FROM recovery_codes WHERE user_id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  sqlx::query("DELETE FROM users WHERE id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn count_sessions(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
  let row: (i64,) =
    sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE user_id = $1 AND expires_at > now()")
      .bind(user_id)
      .fetch_one(pool)
      .await?;
  Ok(row.0)
}

pub async fn count_tokens(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
  let row: (i64,) =
    sqlx::query_as("SELECT COUNT(*) FROM api_tokens WHERE created_by = $1 AND revoked = false")
      .bind(user_id)
      .fetch_one(pool)
      .await?;
  Ok(row.0)
}
