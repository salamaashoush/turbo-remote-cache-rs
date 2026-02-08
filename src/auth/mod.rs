pub mod api_token;
pub mod extractors;
pub mod jwt;
pub mod middleware;
pub mod password;
pub mod recovery;
pub mod totp;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the authenticated identity attached to a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthInfo {
  ApiToken {
    token_id: Uuid,
    org_id: Uuid,
    team_id: Option<Uuid>,
    scopes: Vec<String>,
  },
  Jwt {
    user_id: Uuid,
    email: String,
    role: String,
  },
  LegacyToken,
}

// Re-export the middleware Transform for backward compatibility
pub use middleware::Auth;
