use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CacheEvent {
  pub id: i64,
  pub org_id: Uuid,
  pub team_id: Option<Uuid>,
  pub artifact_hash: String,
  pub event_type: String,
  pub size_bytes: i64,
  pub duration_ms: i32,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsOverview {
  pub period: String,
  pub total_events: i64,
  pub hits: i64,
  pub misses: i64,
  pub puts: i64,
  pub hit_rate: f64,
  pub total_bytes_saved: i64,
  pub estimated_time_saved_ms: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct TimelinePoint {
  pub date: String,
  pub hits: i64,
  pub misses: i64,
  pub puts: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct TeamAnalytics {
  pub team_id: Option<Uuid>,
  pub hits: i64,
  pub misses: i64,
  pub puts: i64,
  pub total_bytes: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct ArtifactEntry {
  pub artifact_hash: String,
  pub last_event: String,
  pub total_events: i64,
  pub total_bytes: i64,
  pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PlatformStats {
  pub total_users: i64,
  pub total_orgs: i64,
  pub total_events: i64,
  pub total_bytes: i64,
  pub total_active_tokens: i64,
  pub total_active_sessions: i64,
  pub total_hits: i64,
  pub total_misses: i64,
  pub total_puts: i64,
  pub total_duration_ms: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct OrgAnalytics {
  pub org_id: Uuid,
  pub org_name: String,
  pub org_slug: String,
  pub hits: i64,
  pub misses: i64,
  pub puts: i64,
  pub total_bytes: i64,
}
