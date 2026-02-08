use sqlx::PgPool;
use std::time::Duration;
use tracing::info;

pub fn spawn_cleanup_jobs(pool: PgPool) {
  // Session cleanup — every hour
  let pool_sessions = pool.clone();
  tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
      interval.tick().await;
      match crate::db::sessions::cleanup_expired(&pool_sessions).await {
        Ok(count) if count > 0 => info!("Cleaned up {} expired sessions", count),
        _ => {}
      }
    }
  });

  // Token expiry cleanup — every hour
  let pool_tokens = pool.clone();
  tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
      interval.tick().await;
      match crate::db::api_tokens::revoke_expired(&pool_tokens).await {
        Ok(count) if count > 0 => info!("Revoked {} expired API tokens", count),
        _ => {}
      }
    }
  });

  // Email token cleanup — every hour
  let pool_email = pool.clone();
  tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
      interval.tick().await;
      match crate::db::email_tokens::cleanup_expired(&pool_email).await {
        Ok(count) if count > 0 => info!("Cleaned up {} expired email tokens", count),
        _ => {}
      }
    }
  });

  // 2FA pending session cleanup — every 15 minutes
  tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(900));
    loop {
      interval.tick().await;
      match crate::db::twofa_pending::cleanup_expired(&pool).await {
        Ok(count) if count > 0 => info!("Cleaned up {} expired 2FA pending sessions", count),
        _ => {}
      }
    }
  });
}
