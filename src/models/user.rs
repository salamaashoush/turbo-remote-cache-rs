use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
  pub id: Uuid,
  pub email: String,
  #[serde(skip_serializing)]
  pub password_hash: String,
  pub name: String,
  pub role: String,
  pub email_verified: bool,
  pub twofa_method: String,
  #[serde(skip_serializing)]
  pub totp_secret: Option<String>,
  pub totp_enabled: bool,
  pub is_active: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserWithDetails {
  pub id: Uuid,
  pub email: String,
  pub name: String,
  pub role: String,
  pub email_verified: bool,
  pub is_active: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub org_name: Option<String>,
  pub last_active: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct AdminUserResponse {
  pub id: Uuid,
  pub email: String,
  pub name: String,
  pub role: String,
  pub email_verified: bool,
  pub is_active: bool,
  pub created_at: DateTime<Utc>,
  pub org_name: Option<String>,
  pub last_active: Option<DateTime<Utc>>,
}

impl From<UserWithDetails> for AdminUserResponse {
  fn from(u: UserWithDetails) -> Self {
    Self {
      id: u.id,
      email: u.email,
      name: u.name,
      role: u.role,
      email_verified: u.email_verified,
      is_active: u.is_active,
      created_at: u.created_at,
      org_name: u.org_name,
      last_active: u.last_active,
    }
  }
}

#[derive(Debug, Serialize)]
pub struct AdminUserOrgEntry {
  pub id: Uuid,
  pub name: String,
  pub slug: String,
  pub role: String,
}

#[derive(Debug, Serialize)]
pub struct AdminUserDetailResponse {
  pub id: Uuid,
  pub email: String,
  pub name: String,
  pub role: String,
  pub is_active: bool,
  pub email_verified: bool,
  pub twofa_method: String,
  pub twofa_enabled: bool,
  pub created_at: DateTime<Utc>,
  pub last_active: Option<DateTime<Utc>>,
  pub orgs: Vec<AdminUserOrgEntry>,
  pub session_count: i64,
  pub token_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct SetActiveRequest {
  pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct AdminAddToOrgRequest {
  pub org_id: Uuid,
  pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
  pub email: String,
  pub password: String,
  pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
  pub email: String,
  pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
  pub access_token: String,
  pub refresh_token: String,
  pub expires_in: i64,
  pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
  pub id: Uuid,
  pub email: String,
  pub name: String,
  pub role: String,
  pub email_verified: bool,
  pub twofa_method: String,
  pub twofa_enabled: bool,
  pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
  fn from(u: User) -> Self {
    let twofa_enabled = u.twofa_method != "none";
    Self {
      id: u.id,
      email: u.email,
      name: u.name,
      role: u.role,
      email_verified: u.email_verified,
      twofa_method: u.twofa_method,
      twofa_enabled,
      created_at: u.created_at,
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
  pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
  pub access_token: String,
  pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
  pub access_token: String,
  pub refresh_token: String,
  pub expires_in: i64,
  pub user: UserResponse,
  pub org: Option<crate::models::organization::Organization>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
  pub name: Option<String>,
  pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
  pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
  pub token: String,
  pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
  pub token: String,
}

// --- 2FA types ---

#[derive(Debug, Serialize)]
pub struct LoginTwofaRequiredResponse {
  pub twofa_required: bool,
  pub twofa_method: String,
  pub pending_token: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyTwofaRequest {
  pub pending_token: String,
  pub code: String,
}

#[derive(Debug, Serialize)]
pub struct TotpSetupResponse {
  pub secret: String,
  pub qr_code: String,
  pub otpauth_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct TotpConfirmRequest {
  pub code: String,
}

#[derive(Debug, Serialize)]
pub struct TwofaEnabledResponse {
  pub recovery_codes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TwofaDisableRequest {
  pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TwofaStatusResponse {
  pub method: String,
  pub enabled: bool,
  pub recovery_codes_remaining: i64,
}

#[derive(Debug, Serialize)]
pub struct RecoveryCodesResponse {
  pub recovery_codes: Vec<String>,
}
