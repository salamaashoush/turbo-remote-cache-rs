use sqlx::PgPool;
use uuid::Uuid;

use crate::models::cache_event::{ArtifactEntry, TeamAnalytics, TimelinePoint};

pub async fn record_event(
  pool: &PgPool,
  org_id: Uuid,
  team_id: Option<Uuid>,
  artifact_hash: &str,
  event_type: &str,
  size_bytes: i64,
  duration_ms: i32,
) -> Result<(), sqlx::Error> {
  sqlx::query(
    "INSERT INTO cache_events (org_id, team_id, artifact_hash, event_type, size_bytes, duration_ms)
     VALUES ($1, $2, $3, $4, $5, $6)",
  )
  .bind(org_id)
  .bind(team_id)
  .bind(artifact_hash)
  .bind(event_type)
  .bind(size_bytes)
  .bind(duration_ms)
  .execute(pool)
  .await?;
  Ok(())
}

#[derive(sqlx::FromRow)]
pub struct EventCounts {
  pub total: i64,
  pub hits: i64,
  pub misses: i64,
  pub puts: i64,
  pub total_bytes: i64,
  pub total_duration_ms: i64,
}

pub async fn get_overview(
  pool: &PgPool,
  org_id: Uuid,
  since: chrono::DateTime<chrono::Utc>,
) -> Result<EventCounts, sqlx::Error> {
  sqlx::query_as::<_, EventCounts>(
    "SELECT
       COUNT(*) as total,
       COUNT(*) FILTER (WHERE event_type = 'hit') as hits,
       COUNT(*) FILTER (WHERE event_type = 'miss') as misses,
       COUNT(*) FILTER (WHERE event_type = 'put') as puts,
       COALESCE(SUM(size_bytes) FILTER (WHERE event_type = 'hit'), 0)::BIGINT as total_bytes,
       COALESCE(SUM(duration_ms) FILTER (WHERE event_type = 'hit'), 0)::BIGINT as total_duration_ms
     FROM cache_events
     WHERE org_id = $1 AND created_at >= $2",
  )
  .bind(org_id)
  .bind(since)
  .fetch_one(pool)
  .await
}

pub async fn get_timeline(
  pool: &PgPool,
  org_id: Uuid,
  since: chrono::DateTime<chrono::Utc>,
  granularity: &str,
) -> Result<Vec<TimelinePoint>, sqlx::Error> {
  let query = match granularity {
    "hour" => {
      "SELECT
         to_char(date_trunc('hour', created_at), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as date,
         COUNT(*) FILTER (WHERE event_type = 'hit') as hits,
         COUNT(*) FILTER (WHERE event_type = 'miss') as misses,
         COUNT(*) FILTER (WHERE event_type = 'put') as puts
       FROM cache_events
       WHERE org_id = $1 AND created_at >= $2
       GROUP BY date_trunc('hour', created_at)
       ORDER BY date_trunc('hour', created_at)"
    }
    "week" => {
      "SELECT
         to_char(date_trunc('week', created_at), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as date,
         COUNT(*) FILTER (WHERE event_type = 'hit') as hits,
         COUNT(*) FILTER (WHERE event_type = 'miss') as misses,
         COUNT(*) FILTER (WHERE event_type = 'put') as puts
       FROM cache_events
       WHERE org_id = $1 AND created_at >= $2
       GROUP BY date_trunc('week', created_at)
       ORDER BY date_trunc('week', created_at)"
    }
    _ => {
      "SELECT
         to_char(date_trunc('day', created_at), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as date,
         COUNT(*) FILTER (WHERE event_type = 'hit') as hits,
         COUNT(*) FILTER (WHERE event_type = 'miss') as misses,
         COUNT(*) FILTER (WHERE event_type = 'put') as puts
       FROM cache_events
       WHERE org_id = $1 AND created_at >= $2
       GROUP BY date_trunc('day', created_at)
       ORDER BY date_trunc('day', created_at)"
    }
  };

  sqlx::query_as::<_, TimelinePoint>(query)
    .bind(org_id)
    .bind(since)
    .fetch_all(pool)
    .await
}

pub async fn get_team_breakdown(
  pool: &PgPool,
  org_id: Uuid,
  since: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<TeamAnalytics>, sqlx::Error> {
  sqlx::query_as::<_, TeamAnalytics>(
    "SELECT
       team_id,
       COUNT(*) FILTER (WHERE event_type = 'hit') as hits,
       COUNT(*) FILTER (WHERE event_type = 'miss') as misses,
       COUNT(*) FILTER (WHERE event_type = 'put') as puts,
       COALESCE(SUM(size_bytes), 0)::BIGINT as total_bytes
     FROM cache_events
     WHERE org_id = $1 AND created_at >= $2
     GROUP BY team_id",
  )
  .bind(org_id)
  .bind(since)
  .fetch_all(pool)
  .await
}

pub async fn list_artifacts(
  pool: &PgPool,
  org_id: Uuid,
  limit: i64,
  offset: i64,
) -> Result<Vec<ArtifactEntry>, sqlx::Error> {
  sqlx::query_as::<_, ArtifactEntry>(
    "SELECT
       artifact_hash,
       (array_agg(event_type ORDER BY created_at DESC))[1] as last_event,
       COUNT(*) as total_events,
       COALESCE(SUM(size_bytes), 0)::BIGINT as total_bytes,
       MAX(created_at) as last_seen
     FROM cache_events
     WHERE org_id = $1
     GROUP BY artifact_hash
     ORDER BY last_seen DESC
     LIMIT $2 OFFSET $3",
  )
  .bind(org_id)
  .bind(limit)
  .bind(offset)
  .fetch_all(pool)
  .await
}

pub async fn get_total_events(pool: &PgPool) -> Result<i64, sqlx::Error> {
  let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cache_events")
    .fetch_one(pool)
    .await?;
  Ok(row.0)
}

pub async fn get_total_bytes(pool: &PgPool) -> Result<i64, sqlx::Error> {
  let row: (i64,) = sqlx::query_as("SELECT COALESCE(SUM(size_bytes), 0)::BIGINT FROM cache_events")
    .fetch_one(pool)
    .await?;
  Ok(row.0)
}

pub async fn get_platform_overview(
  pool: &PgPool,
  since: chrono::DateTime<chrono::Utc>,
) -> Result<EventCounts, sqlx::Error> {
  sqlx::query_as::<_, EventCounts>(
    "SELECT
       COUNT(*) as total,
       COUNT(*) FILTER (WHERE event_type = 'hit') as hits,
       COUNT(*) FILTER (WHERE event_type = 'miss') as misses,
       COUNT(*) FILTER (WHERE event_type = 'put') as puts,
       COALESCE(SUM(size_bytes) FILTER (WHERE event_type = 'hit'), 0)::BIGINT as total_bytes,
       COALESCE(SUM(duration_ms) FILTER (WHERE event_type = 'hit'), 0)::BIGINT as total_duration_ms
     FROM cache_events
     WHERE created_at >= $1",
  )
  .bind(since)
  .fetch_one(pool)
  .await
}

pub async fn get_platform_timeline(
  pool: &PgPool,
  since: chrono::DateTime<chrono::Utc>,
  granularity: &str,
) -> Result<Vec<TimelinePoint>, sqlx::Error> {
  let query = match granularity {
    "hour" => {
      "SELECT
         to_char(date_trunc('hour', created_at), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as date,
         COUNT(*) FILTER (WHERE event_type = 'hit') as hits,
         COUNT(*) FILTER (WHERE event_type = 'miss') as misses,
         COUNT(*) FILTER (WHERE event_type = 'put') as puts
       FROM cache_events
       WHERE created_at >= $1
       GROUP BY date_trunc('hour', created_at)
       ORDER BY date_trunc('hour', created_at)"
    }
    "week" => {
      "SELECT
         to_char(date_trunc('week', created_at), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as date,
         COUNT(*) FILTER (WHERE event_type = 'hit') as hits,
         COUNT(*) FILTER (WHERE event_type = 'miss') as misses,
         COUNT(*) FILTER (WHERE event_type = 'put') as puts
       FROM cache_events
       WHERE created_at >= $1
       GROUP BY date_trunc('week', created_at)
       ORDER BY date_trunc('week', created_at)"
    }
    _ => {
      "SELECT
         to_char(date_trunc('day', created_at), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as date,
         COUNT(*) FILTER (WHERE event_type = 'hit') as hits,
         COUNT(*) FILTER (WHERE event_type = 'miss') as misses,
         COUNT(*) FILTER (WHERE event_type = 'put') as puts
       FROM cache_events
       WHERE created_at >= $1
       GROUP BY date_trunc('day', created_at)
       ORDER BY date_trunc('day', created_at)"
    }
  };

  sqlx::query_as::<_, TimelinePoint>(query)
    .bind(since)
    .fetch_all(pool)
    .await
}

use crate::models::cache_event::OrgAnalytics;

pub async fn total_bytes_by_org(pool: &PgPool, org_id: Uuid) -> Result<i64, sqlx::Error> {
  let row: (i64,) = sqlx::query_as(
    "SELECT COALESCE(SUM(size_bytes), 0)::BIGINT FROM cache_events WHERE org_id = $1",
  )
  .bind(org_id)
  .fetch_one(pool)
  .await?;
  Ok(row.0)
}

pub async fn total_events_by_org(pool: &PgPool, org_id: Uuid) -> Result<i64, sqlx::Error> {
  let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cache_events WHERE org_id = $1")
    .bind(org_id)
    .fetch_one(pool)
    .await?;
  Ok(row.0)
}

pub async fn get_org_breakdown(
  pool: &PgPool,
  since: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<OrgAnalytics>, sqlx::Error> {
  sqlx::query_as::<_, OrgAnalytics>(
    "SELECT
       ce.org_id,
       o.name as org_name,
       o.slug as org_slug,
       COUNT(*) FILTER (WHERE ce.event_type = 'hit') as hits,
       COUNT(*) FILTER (WHERE ce.event_type = 'miss') as misses,
       COUNT(*) FILTER (WHERE ce.event_type = 'put') as puts,
       COALESCE(SUM(ce.size_bytes), 0)::BIGINT as total_bytes
     FROM cache_events ce
     JOIN organizations o ON ce.org_id = o.id
     WHERE ce.created_at >= $1
     GROUP BY ce.org_id, o.name, o.slug",
  )
  .bind(since)
  .fetch_all(pool)
  .await
}
