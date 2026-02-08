use sqlx::PgPool;
use uuid::Uuid;

use crate::models::team::{TeamMember, TeamMemberWithEmail};

pub async fn add_member(
  pool: &PgPool,
  team_id: Uuid,
  user_id: Uuid,
  role: &str,
) -> Result<TeamMember, sqlx::Error> {
  sqlx::query_as::<_, TeamMember>(
    "INSERT INTO team_members (team_id, user_id, role) VALUES ($1, $2, $3) RETURNING *",
  )
  .bind(team_id)
  .bind(user_id)
  .bind(role)
  .fetch_one(pool)
  .await
}

pub async fn list_members(
  pool: &PgPool,
  team_id: Uuid,
) -> Result<Vec<TeamMemberWithEmail>, sqlx::Error> {
  sqlx::query_as::<_, TeamMemberWithEmail>(
    "SELECT tm.team_id, tm.user_id, tm.role, u.email, u.name, tm.added_at
     FROM team_members tm
     INNER JOIN users u ON tm.user_id = u.id
     WHERE tm.team_id = $1
     ORDER BY tm.added_at",
  )
  .bind(team_id)
  .fetch_all(pool)
  .await
}

pub async fn get_member_role(
  pool: &PgPool,
  team_id: Uuid,
  user_id: Uuid,
) -> Result<Option<TeamMember>, sqlx::Error> {
  sqlx::query_as::<_, TeamMember>("SELECT * FROM team_members WHERE team_id = $1 AND user_id = $2")
    .bind(team_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn remove_member(pool: &PgPool, team_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM team_members WHERE team_id = $1 AND user_id = $2")
    .bind(team_id)
    .bind(user_id)
    .execute(pool)
    .await?;
  Ok(())
}
