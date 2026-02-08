use sqlx::PgPool;
use uuid::Uuid;

use crate::models::organization::Organization;

pub async fn create(
  pool: &PgPool,
  name: &str,
  slug: &str,
  owner_id: Uuid,
) -> Result<Organization, sqlx::Error> {
  sqlx::query_as::<_, Organization>(
    "INSERT INTO organizations (name, slug, owner_id) VALUES ($1, $2, $3) RETURNING *",
  )
  .bind(name)
  .bind(slug)
  .bind(owner_id)
  .fetch_one(pool)
  .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Organization>, sqlx::Error> {
  sqlx::query_as::<_, Organization>("SELECT * FROM organizations WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Organization>, sqlx::Error> {
  sqlx::query_as::<_, Organization>("SELECT * FROM organizations WHERE slug = $1")
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Organization>, sqlx::Error> {
  sqlx::query_as::<_, Organization>(
    "SELECT o.* FROM organizations o
     INNER JOIN org_members om ON o.id = om.org_id
     WHERE om.user_id = $1
     ORDER BY o.created_at DESC",
  )
  .bind(user_id)
  .fetch_all(pool)
  .await
}

pub async fn update(
  pool: &PgPool,
  id: Uuid,
  name: Option<&str>,
  slug: Option<&str>,
) -> Result<Organization, sqlx::Error> {
  let org = find_by_id(pool, id)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;

  let name = name.unwrap_or(&org.name);
  let slug = slug.unwrap_or(&org.slug);

  sqlx::query_as::<_, Organization>(
    "UPDATE organizations SET name = $1, slug = $2 WHERE id = $3 RETURNING *",
  )
  .bind(name)
  .bind(slug)
  .bind(id)
  .fetch_one(pool)
  .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM organizations WHERE id = $1")
    .bind(id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn list_all(
  pool: &PgPool,
  limit: i64,
  offset: i64,
) -> Result<Vec<Organization>, sqlx::Error> {
  sqlx::query_as::<_, Organization>(
    "SELECT * FROM organizations ORDER BY created_at DESC LIMIT $1 OFFSET $2",
  )
  .bind(limit)
  .bind(offset)
  .fetch_all(pool)
  .await
}

pub async fn list_all_with_counts(
  pool: &PgPool,
  limit: i64,
  offset: i64,
) -> Result<Vec<crate::models::organization::OrgWithCounts>, sqlx::Error> {
  sqlx::query_as::<_, crate::models::organization::OrgWithCounts>(
    "SELECT o.id, o.name, o.slug, o.owner_id, o.storage_provider, o.storage_config,
            o.cache_size_limit_bytes, o.max_tokens,
            o.created_at, o.updated_at,
            u.email as owner_email,
            (SELECT COUNT(*) FROM org_members WHERE org_id = o.id) as member_count,
            (SELECT COUNT(*) FROM teams WHERE org_id = o.id) as team_count
     FROM organizations o
     JOIN users u ON o.owner_id = u.id
     ORDER BY o.created_at DESC
     LIMIT $1 OFFSET $2",
  )
  .bind(limit)
  .bind(offset)
  .fetch_all(pool)
  .await
}

pub async fn count_all(pool: &PgPool) -> Result<i64, sqlx::Error> {
  let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM organizations")
    .fetch_one(pool)
    .await?;
  Ok(row.0)
}

pub async fn update_limits(
  pool: &PgPool,
  id: Uuid,
  cache_size_limit_bytes: Option<i64>,
  max_tokens: Option<i32>,
) -> Result<Organization, sqlx::Error> {
  sqlx::query_as::<_, Organization>(
    "UPDATE organizations SET cache_size_limit_bytes = $1, max_tokens = $2 WHERE id = $3 RETURNING *",
  )
  .bind(cache_size_limit_bytes)
  .bind(max_tokens)
  .bind(id)
  .fetch_one(pool)
  .await
}

pub async fn transfer_ownership(
  pool: &PgPool,
  id: Uuid,
  new_owner_id: Uuid,
) -> Result<Organization, sqlx::Error> {
  sqlx::query_as::<_, Organization>(
    "UPDATE organizations SET owner_id = $1 WHERE id = $2 RETURNING *",
  )
  .bind(new_owner_id)
  .bind(id)
  .fetch_one(pool)
  .await
}
