use sqlx::PgPool;
use uuid::Uuid;

use crate::models::team::Team;

pub async fn create(
  pool: &PgPool,
  org_id: Uuid,
  name: &str,
  slug: &str,
) -> Result<Team, sqlx::Error> {
  sqlx::query_as::<_, Team>(
    "INSERT INTO teams (org_id, name, slug) VALUES ($1, $2, $3) RETURNING *",
  )
  .bind(org_id)
  .bind(name)
  .bind(slug)
  .fetch_one(pool)
  .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Team>, sqlx::Error> {
  sqlx::query_as::<_, Team>("SELECT * FROM teams WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn list_by_org(pool: &PgPool, org_id: Uuid) -> Result<Vec<Team>, sqlx::Error> {
  sqlx::query_as::<_, Team>("SELECT * FROM teams WHERE org_id = $1 ORDER BY created_at DESC")
    .bind(org_id)
    .fetch_all(pool)
    .await
}

pub async fn update(
  pool: &PgPool,
  id: Uuid,
  name: Option<&str>,
  slug: Option<&str>,
) -> Result<Team, sqlx::Error> {
  let team = find_by_id(pool, id)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;

  let name = name.unwrap_or(&team.name);
  let slug = slug.unwrap_or(&team.slug);

  sqlx::query_as::<_, Team>("UPDATE teams SET name = $1, slug = $2 WHERE id = $3 RETURNING *")
    .bind(name)
    .bind(slug)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM teams WHERE id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  Ok(())
}
