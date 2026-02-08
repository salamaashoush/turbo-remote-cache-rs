use actix_web::HttpResponse;
use actix_web::web::{Data, Query};
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;

use crate::auth::extractors::OrgMember;
use crate::db;
use crate::models::cache_event::AnalyticsOverview;

#[derive(Deserialize)]
pub struct AnalyticsQuery {
  pub period: Option<String>,
  pub granularity: Option<String>,
}

fn parse_period(period: &str) -> Duration {
  match period {
    "1d" => Duration::days(1),
    "7d" => Duration::days(7),
    "30d" => Duration::days(30),
    "90d" => Duration::days(90),
    _ => Duration::days(30),
  }
}

pub async fn overview(
  member: OrgMember,
  pool: Data<PgPool>,
  query: Query<AnalyticsQuery>,
) -> HttpResponse {
  let period = query.period.as_deref().unwrap_or("30d");
  let since = Utc::now() - parse_period(period);

  match db::cache_events::get_overview(&pool, member.org_id, since).await {
    Ok(counts) => {
      let hit_rate = if counts.total > 0 {
        counts.hits as f64 / counts.total as f64
      } else {
        0.0
      };

      HttpResponse::Ok().json(AnalyticsOverview {
        period: period.to_string(),
        total_events: counts.total,
        hits: counts.hits,
        misses: counts.misses,
        puts: counts.puts,
        hit_rate,
        total_bytes_saved: counts.total_bytes,
        estimated_time_saved_ms: counts.total_duration_ms,
      })
    }
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to get analytics: {e}")
    })),
  }
}

pub async fn timeline(
  member: OrgMember,
  pool: Data<PgPool>,
  query: Query<AnalyticsQuery>,
) -> HttpResponse {
  let period = query.period.as_deref().unwrap_or("30d");
  let granularity = query.granularity.as_deref().unwrap_or("day");
  let since = Utc::now() - parse_period(period);

  match db::cache_events::get_timeline(&pool, member.org_id, since, granularity).await {
    Ok(points) => HttpResponse::Ok().json(points),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to get timeline: {e}")
    })),
  }
}

pub async fn teams_breakdown(
  member: OrgMember,
  pool: Data<PgPool>,
  query: Query<AnalyticsQuery>,
) -> HttpResponse {
  let period = query.period.as_deref().unwrap_or("30d");
  let since = Utc::now() - parse_period(period);

  match db::cache_events::get_team_breakdown(&pool, member.org_id, since).await {
    Ok(breakdown) => HttpResponse::Ok().json(breakdown),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to get team breakdown: {e}")
    })),
  }
}
