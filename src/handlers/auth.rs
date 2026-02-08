use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Json};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use tracing::warn;

use crate::auth::api_token::{generate_token, hash_token};
use crate::auth::extractors::AuthenticatedUser;
use crate::auth::jwt::create_access_token;
use crate::auth::password::{hash_password, verify_password};
use crate::config::Config;
use crate::db;
use crate::email::Mailer;
use crate::models::organization::Organization;
use crate::models::user::{
  AuthResponse, ForgotPasswordRequest, LoginRequest, LoginTwofaRequiredResponse, RefreshRequest,
  RegisterRequest, RegisterResponse, ResetPasswordRequest, TokenResponse, TotpConfirmRequest,
  TotpSetupResponse, TwofaDisableRequest, TwofaEnabledResponse, TwofaStatusResponse,
  UpdateProfileRequest, UserResponse, VerifyEmailRequest, VerifyTwofaRequest,
};

fn is_valid_email(email: &str) -> bool {
  if email.len() > 255 {
    return false;
  }
  let parts: Vec<&str> = email.splitn(2, '@').collect();
  if parts.len() != 2 {
    return false;
  }
  let local = parts[0];
  let domain = parts[1];
  !local.is_empty() && !domain.is_empty() && domain.contains('.')
}

fn validate_password(password: &str) -> Result<(), &'static str> {
  if password.len() < 10 {
    return Err("Password must be at least 10 characters");
  }
  if !password.chars().any(|c| c.is_ascii_uppercase()) {
    return Err("Password must contain at least one uppercase letter");
  }
  if !password.chars().any(|c| c.is_ascii_lowercase()) {
    return Err("Password must contain at least one lowercase letter");
  }
  if !password.chars().any(|c| c.is_ascii_digit()) {
    return Err("Password must contain at least one digit");
  }
  if !password.chars().any(|c| !c.is_ascii_alphanumeric()) {
    return Err("Password must contain at least one special character");
  }
  Ok(())
}

fn sanitize_slug(input: &str) -> String {
  input
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() || c == '-' {
        c.to_ascii_lowercase()
      } else {
        '-'
      }
    })
    .collect::<String>()
    .trim_matches('-')
    .to_string()
}

async fn auto_create_personal_org(
  pool: &PgPool,
  user: &crate::models::user::User,
) -> Option<Organization> {
  let email_prefix = user.email.split('@').next().unwrap_or("user");
  let mut slug = sanitize_slug(email_prefix);
  if slug.is_empty() {
    slug = "user".to_string();
  }

  // If slug taken, append first 8 chars of user UUID
  if let Ok(Some(_)) = db::organizations::find_by_slug(pool, &slug).await {
    let suffix = &user.id.to_string()[..8];
    slug = format!("{}-{}", slug, suffix);
  }

  let display_name = if user.name.is_empty() {
    email_prefix.to_string()
  } else {
    user.name.clone()
  };
  let org_name = format!("{}'s Org", display_name);

  let org = match db::organizations::create(pool, &org_name, &slug, user.id).await {
    Ok(o) => o,
    Err(e) => {
      warn!("Failed to auto-create org for user {}: {}", user.id, e);
      return None;
    }
  };

  // Add user as org admin
  if let Err(e) = db::org_members::add_member(pool, org.id, user.id, "admin").await {
    warn!("Failed to add user as org admin: {}", e);
  }

  // Create default team
  match db::teams::create(pool, org.id, "Default", "default").await {
    Ok(team) => {
      if let Err(e) = db::team_members::add_member(pool, team.id, user.id, "admin").await {
        warn!("Failed to add user as team admin: {}", e);
      }
    }
    Err(e) => {
      warn!("Failed to create default team: {}", e);
    }
  }

  Some(org)
}

/// Helper: issue JWT + refresh token for a user (used after successful auth)
async fn issue_tokens(
  pool: &PgPool,
  user: &crate::models::user::User,
  config: &Config,
) -> Result<(String, String), HttpResponse> {
  let access_token = create_access_token(
    user.id,
    &user.email,
    &user.role,
    &config.jwt_secret,
    config.jwt_access_expiry_secs,
  )
  .map_err(|_| {
    HttpResponse::InternalServerError().json(serde_json::json!({
      "error": "Failed to create access token"
    }))
  })?;

  let refresh_token = generate_token();
  let refresh_hash = hash_token(&refresh_token);
  let refresh_expires = Utc::now() + Duration::seconds(config.jwt_refresh_expiry_secs);

  db::sessions::create(pool, user.id, &refresh_hash, refresh_expires)
    .await
    .map_err(|e| {
      HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Failed to create session: {e}")
      }))
    })?;

  Ok((access_token, refresh_token))
}

pub async fn register(
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
  mailer: Data<Option<Mailer>>,
  body: Json<RegisterRequest>,
) -> HttpResponse {
  if !config.allow_registration {
    return HttpResponse::Forbidden().json(serde_json::json!({
      "error": "Registration is disabled"
    }));
  }

  let name = body.name.as_deref().unwrap_or("");

  if body.email.is_empty() || body.password.is_empty() {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": "Email and password are required"
    }));
  }

  if !is_valid_email(&body.email) {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": "Invalid email address"
    }));
  }

  if let Err(msg) = validate_password(&body.password) {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": msg
    }));
  }

  // Check if user already exists
  if let Ok(Some(_)) = db::users::find_by_email(&pool, &body.email).await {
    return HttpResponse::Conflict().json(serde_json::json!({
      "error": "Email already registered"
    }));
  }

  let password_hash = match hash_password(&body.password) {
    Ok(h) => h,
    Err(_) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": "Failed to hash password"
      }));
    }
  };

  // All registered users get "user" role — super admin is bootstrapped from env vars
  let role = "user";

  let user = match db::users::create_user(&pool, &body.email, &password_hash, name, role).await {
    Ok(u) => u,
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Failed to create user: {e}")
      }));
    }
  };

  // Auto-create personal org + default team
  let org = auto_create_personal_org(&pool, &user).await;

  // Send welcome + verification emails if SMTP configured
  if let Some(mailer) = mailer.as_ref() {
    let mailer = mailer.clone();
    let app_url = config.app_url.clone();
    let user_email = user.email.clone();
    let user_name = user.name.clone();
    let user_id = user.id;
    let pool_clone = pool.get_ref().clone();

    tokio::spawn(async move {
      // Send welcome email
      let (subj, html) = crate::email::templates::welcome_email(&user_name, &app_url);
      if let Err(e) = mailer.send(&user_email, &subj, &html).await {
        warn!("Failed to send welcome email to {}: {}", user_email, e);
      }

      // Generate verification token and send verification email
      let raw_token = generate_token();
      let token_hash = hash_token(&raw_token);
      let expires = Utc::now() + Duration::hours(24);
      if let Err(e) =
        crate::db::email_tokens::create(&pool_clone, user_id, &token_hash, "verify_email", expires)
          .await
      {
        warn!("Failed to create verification token: {}", e);
        return;
      }
      let verify_url = format!("{}/verify-email?token={}", app_url, raw_token);
      let (subj, html) = crate::email::templates::email_verification(&user_name, &verify_url);
      if let Err(e) = mailer.send(&user_email, &subj, &html).await {
        warn!("Failed to send verification email to {}: {}", user_email, e);
      }
    });
  }

  let (access_token, refresh_token) = match issue_tokens(&pool, &user, &config).await {
    Ok(t) => t,
    Err(resp) => return resp,
  };

  HttpResponse::Created().json(RegisterResponse {
    access_token,
    refresh_token,
    expires_in: config.jwt_access_expiry_secs,
    user: UserResponse::from(user),
    org,
  })
}

pub async fn login(
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
  mailer: Data<Option<Mailer>>,
  body: Json<LoginRequest>,
) -> HttpResponse {
  let user = match db::users::find_by_email(&pool, &body.email).await {
    Ok(Some(u)) => u,
    _ => {
      return HttpResponse::Unauthorized().json(serde_json::json!({
        "error": "Invalid email or password"
      }));
    }
  };

  if !verify_password(&body.password, &user.password_hash) {
    warn!("Failed login attempt for email={}", body.email);
    return HttpResponse::Unauthorized().json(serde_json::json!({
      "error": "Invalid email or password"
    }));
  }

  // Check if user is active
  if !user.is_active {
    warn!("Login attempt on deactivated account email={}", body.email);
    return HttpResponse::Forbidden().json(serde_json::json!({
      "error": "Account deactivated"
    }));
  }

  // Check email verification if required
  if config.require_email_verification && !user.email_verified {
    return HttpResponse::Forbidden().json(serde_json::json!({
      "error": "Please verify your email before logging in"
    }));
  }

  // Check if 2FA is enabled
  if user.twofa_method != "none" {
    // Clean up any existing pending 2FA sessions for this user
    let _ = db::twofa_pending::delete_by_user(&pool, user.id).await;

    // Create a pending 2FA token
    let raw_token = generate_token();
    let pending_hash = hash_token(&raw_token);
    let expires = Utc::now() + Duration::minutes(5);

    if let Err(e) =
      db::twofa_pending::create(&pool, user.id, &pending_hash, &user.twofa_method, expires).await
    {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Failed to create 2FA challenge: {e}")
      }));
    }

    // For email method, send OTP
    if user.twofa_method == "email"
      && let Some(mailer) = mailer.as_ref()
    {
      let otp = generate_email_otp();
      // Store the OTP as an email token for verification
      let otp_hash = hash_token(&otp);
      let otp_expires = Utc::now() + Duration::minutes(10);
      let _ =
        db::email_tokens::create(&pool, user.id, &otp_hash, "twofa_email_otp", otp_expires).await;

      let mailer = mailer.clone();
      let email = user.email.clone();
      let name = user.name.clone();
      tokio::spawn(async move {
        let (subj, html) = crate::email::templates::twofa_email_otp(&name, &otp);
        if let Err(e) = mailer.send(&email, &subj, &html).await {
          warn!("Failed to send 2FA OTP email to {}: {}", email, e);
        }
      });
    }

    return HttpResponse::Ok().json(LoginTwofaRequiredResponse {
      twofa_required: true,
      twofa_method: user.twofa_method.clone(),
      pending_token: raw_token,
    });
  }

  // No 2FA — issue tokens directly
  let (access_token, refresh_token) = match issue_tokens(&pool, &user, &config).await {
    Ok(t) => t,
    Err(resp) => return resp,
  };

  HttpResponse::Ok().json(AuthResponse {
    access_token,
    refresh_token,
    expires_in: config.jwt_access_expiry_secs,
    user: UserResponse::from(user),
  })
}

/// Generate a 6-digit numeric OTP for email 2FA
fn generate_email_otp() -> String {
  let bytes: [u8; 4] = rand::random();
  let num = u32::from_le_bytes(bytes) % 1_000_000;
  format!("{:06}", num)
}

pub async fn verify_twofa_login(
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
  body: Json<VerifyTwofaRequest>,
) -> HttpResponse {
  let pending_hash = hash_token(&body.pending_token);

  let pending = match db::twofa_pending::find_by_hash(&pool, &pending_hash).await {
    Ok(Some(p)) => p,
    _ => {
      return HttpResponse::Unauthorized().json(serde_json::json!({
        "error": "Invalid or expired 2FA session"
      }));
    }
  };

  let user = match db::users::find_by_id(&pool, pending.user_id).await {
    Ok(Some(u)) => u,
    _ => {
      return HttpResponse::Unauthorized().json(serde_json::json!({
        "error": "User not found"
      }));
    }
  };

  let code_valid = match pending.method.as_str() {
    "totp" => {
      // Verify TOTP code
      match &user.totp_secret {
        Some(secret) => {
          crate::auth::totp::verify_code(secret, &user.email, &body.code).unwrap_or(false)
        }
        None => false,
      }
    }
    "email" => {
      // Verify email OTP — check against stored email_tokens
      let code_hash = hash_token(&body.code);
      match db::email_tokens::find_by_hash(&pool, &code_hash, "twofa_email_otp").await {
        Ok(Some(token)) if token.user_id == user.id => {
          let _ = db::email_tokens::delete(&pool, token.id).await;
          true
        }
        _ => false,
      }
    }
    _ => false,
  };

  // If code is not valid, try recovery code as fallback
  let used_recovery = if !code_valid {
    let recovery_hash = crate::auth::recovery::sha256_hash(&body.code);
    match db::recovery_codes::find_unused_by_hash(&pool, user.id, &recovery_hash).await {
      Ok(Some(rc)) => {
        let _ = db::recovery_codes::mark_used(&pool, rc.id).await;
        true
      }
      _ => false,
    }
  } else {
    false
  };

  if !code_valid && !used_recovery {
    warn!("Failed 2FA attempt for user_id={}", user.id);
    return HttpResponse::Unauthorized().json(serde_json::json!({
      "error": "Invalid verification code"
    }));
  }

  // Delete the pending 2FA session
  let _ = db::twofa_pending::delete(&pool, pending.id).await;

  // Issue real tokens
  let (access_token, refresh_token) = match issue_tokens(&pool, &user, &config).await {
    Ok(t) => t,
    Err(resp) => return resp,
  };

  HttpResponse::Ok().json(AuthResponse {
    access_token,
    refresh_token,
    expires_in: config.jwt_access_expiry_secs,
    user: UserResponse::from(user),
  })
}

pub async fn refresh(
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
  body: Json<RefreshRequest>,
) -> HttpResponse {
  let token_hash = hash_token(&body.refresh_token);

  let session = match db::sessions::find_by_hash(&pool, &token_hash).await {
    Ok(Some(s)) => s,
    _ => {
      return HttpResponse::Unauthorized().json(serde_json::json!({
        "error": "Invalid or expired refresh token"
      }));
    }
  };

  let user = match db::users::find_by_id(&pool, session.user_id).await {
    Ok(Some(u)) => u,
    _ => {
      return HttpResponse::Unauthorized().json(serde_json::json!({
        "error": "User not found"
      }));
    }
  };

  if !user.is_active {
    return HttpResponse::Forbidden().json(serde_json::json!({
      "error": "Account deactivated"
    }));
  }

  let access_token = match create_access_token(
    user.id,
    &user.email,
    &user.role,
    &config.jwt_secret,
    config.jwt_access_expiry_secs,
  ) {
    Ok(t) => t,
    Err(_) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": "Failed to create access token"
      }));
    }
  };

  HttpResponse::Ok().json(TokenResponse {
    access_token,
    expires_in: config.jwt_access_expiry_secs,
  })
}

pub async fn logout(pool: Data<PgPool>, auth: AuthenticatedUser) -> HttpResponse {
  let _ = db::sessions::delete_by_user(&pool, auth.0.sub).await;
  HttpResponse::Ok().json(serde_json::json!({ "message": "Logged out" }))
}

pub async fn me(auth: AuthenticatedUser, pool: Data<PgPool>) -> HttpResponse {
  match db::users::find_by_id(&pool, auth.0.sub).await {
    Ok(Some(user)) => HttpResponse::Ok().json(UserResponse::from(user)),
    _ => HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" })),
  }
}

pub async fn update_profile(
  auth: AuthenticatedUser,
  pool: Data<PgPool>,
  body: Json<UpdateProfileRequest>,
) -> HttpResponse {
  let password_hash = match &body.password {
    Some(pw) => {
      if let Err(msg) = validate_password(pw) {
        return HttpResponse::BadRequest().json(serde_json::json!({
          "error": msg
        }));
      }
      match hash_password(pw) {
        Ok(h) => Some(h),
        Err(_) => {
          return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to hash password"
          }));
        }
      }
    }
    None => None,
  };

  match db::users::update_user(
    &pool,
    auth.0.sub,
    body.name.as_deref(),
    password_hash.as_deref(),
  )
  .await
  {
    Ok(user) => HttpResponse::Ok().json(UserResponse::from(user)),
    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to update profile: {e}")
    })),
  }
}

pub async fn verify_email(pool: Data<PgPool>, body: Json<VerifyEmailRequest>) -> HttpResponse {
  let token_hash = hash_token(&body.token);

  let email_token = match db::email_tokens::find_by_hash(&pool, &token_hash, "verify_email").await {
    Ok(Some(t)) => t,
    Ok(None) => {
      return HttpResponse::BadRequest().json(serde_json::json!({
        "error": "Invalid or expired verification token"
      }));
    }
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Database error: {e}")
      }));
    }
  };

  if let Err(e) = db::users::set_email_verified(&pool, email_token.user_id).await {
    return HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to verify email: {e}")
    }));
  }

  // Delete the used token
  let _ = db::email_tokens::delete(&pool, email_token.id).await;

  HttpResponse::Ok().json(serde_json::json!({ "message": "Email verified successfully" }))
}

pub async fn forgot_password(
  pool: Data<PgPool>,
  config: Data<Arc<Config>>,
  mailer: Data<Option<Mailer>>,
  body: Json<ForgotPasswordRequest>,
) -> HttpResponse {
  // Always return 200 to prevent email enumeration
  let ok = serde_json::json!({ "message": "If that email exists, a reset link has been sent" });

  let mailer = match mailer.as_ref() {
    Some(m) => m.clone(),
    None => return HttpResponse::Ok().json(&ok),
  };

  let user = match db::users::find_by_email(&pool, &body.email).await {
    Ok(Some(u)) => u,
    _ => return HttpResponse::Ok().json(&ok),
  };

  // Delete any existing reset tokens for this user
  let _ = db::email_tokens::delete_by_user(&pool, user.id, "reset_password").await;

  let raw_token = generate_token();
  let token_hash = hash_token(&raw_token);
  let expires = Utc::now() + Duration::hours(1);

  if let Err(e) =
    db::email_tokens::create(&pool, user.id, &token_hash, "reset_password", expires).await
  {
    warn!("Failed to create reset token: {}", e);
    return HttpResponse::Ok().json(&ok);
  }

  let reset_url = format!("{}/reset-password?token={}", config.app_url, raw_token);
  let (subj, html) = crate::email::templates::password_reset(&user.name, &reset_url);

  let email = user.email.clone();
  tokio::spawn(async move {
    if let Err(e) = mailer.send(&email, &subj, &html).await {
      warn!("Failed to send reset email to {}: {}", email, e);
    }
  });

  HttpResponse::Ok().json(&ok)
}

pub async fn reset_password(pool: Data<PgPool>, body: Json<ResetPasswordRequest>) -> HttpResponse {
  if let Err(msg) = validate_password(&body.new_password) {
    return HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }));
  }

  let token_hash = hash_token(&body.token);

  let email_token = match db::email_tokens::find_by_hash(&pool, &token_hash, "reset_password").await
  {
    Ok(Some(t)) => t,
    Ok(None) => {
      return HttpResponse::BadRequest().json(serde_json::json!({
        "error": "Invalid or expired reset token"
      }));
    }
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Database error: {e}")
      }));
    }
  };

  let password_hash = match hash_password(&body.new_password) {
    Ok(h) => h,
    Err(_) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": "Failed to hash password"
      }));
    }
  };

  if let Err(e) = db::users::update_password(&pool, email_token.user_id, &password_hash).await {
    return HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to update password: {e}")
    }));
  }

  // Delete the used token
  let _ = db::email_tokens::delete(&pool, email_token.id).await;

  // Invalidate all sessions and pending 2FA challenges
  let _ = db::sessions::delete_by_user(&pool, email_token.user_id).await;
  let _ = db::twofa_pending::delete_by_user(&pool, email_token.user_id).await;

  HttpResponse::Ok().json(serde_json::json!({ "message": "Password reset successfully" }))
}

// --- 2FA Management Handlers ---

pub async fn setup_totp(auth: AuthenticatedUser, pool: Data<PgPool>) -> HttpResponse {
  let user = match db::users::find_by_id(&pool, auth.0.sub).await {
    Ok(Some(u)) => u,
    _ => {
      return HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" }));
    }
  };

  let secret = crate::auth::totp::generate_secret();
  let totp = match crate::auth::totp::build_totp(&secret, &user.email) {
    Ok(t) => t,
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Failed to generate TOTP: {e}")
      }));
    }
  };

  let qr_code = match crate::auth::totp::generate_qr_base64(&totp) {
    Ok(q) => q,
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("Failed to generate QR code: {e}")
      }));
    }
  };

  let otpauth_uri = crate::auth::totp::get_otpauth_uri(&totp);

  // Store secret on user (not yet enabled)
  if let Err(e) = db::users::set_totp_secret(&pool, user.id, &secret).await {
    return HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to save TOTP secret: {e}")
    }));
  }

  HttpResponse::Ok().json(TotpSetupResponse {
    secret,
    qr_code,
    otpauth_uri,
  })
}

pub async fn confirm_totp(
  auth: AuthenticatedUser,
  pool: Data<PgPool>,
  body: Json<TotpConfirmRequest>,
) -> HttpResponse {
  let user = match db::users::find_by_id(&pool, auth.0.sub).await {
    Ok(Some(u)) => u,
    _ => {
      return HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" }));
    }
  };

  let secret = match &user.totp_secret {
    Some(s) => s.clone(),
    None => {
      return HttpResponse::BadRequest().json(serde_json::json!({
        "error": "TOTP not set up — call setup first"
      }));
    }
  };

  let valid = match crate::auth::totp::verify_code(&secret, &user.email, &body.code) {
    Ok(v) => v,
    Err(e) => {
      return HttpResponse::InternalServerError().json(serde_json::json!({
        "error": format!("TOTP verification failed: {e}")
      }));
    }
  };

  if !valid {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": "Invalid TOTP code"
    }));
  }

  // Enable TOTP
  if let Err(e) = db::users::enable_totp(&pool, user.id).await {
    return HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to enable TOTP: {e}")
    }));
  }

  // Generate recovery codes
  let codes = crate::auth::recovery::generate_recovery_codes();
  let plaintexts: Vec<String> = codes.iter().map(|(p, _)| p.clone()).collect();
  let hashes: Vec<String> = codes.iter().map(|(_, h)| h.clone()).collect();

  // Delete any existing recovery codes and create new ones
  let _ = db::recovery_codes::delete_by_user(&pool, user.id).await;
  if let Err(e) = db::recovery_codes::create_batch(&pool, user.id, &hashes).await {
    return HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to save recovery codes: {e}")
    }));
  }

  HttpResponse::Ok().json(TwofaEnabledResponse {
    recovery_codes: plaintexts,
  })
}

pub async fn enable_email_twofa(auth: AuthenticatedUser, pool: Data<PgPool>) -> HttpResponse {
  let user = match db::users::find_by_id(&pool, auth.0.sub).await {
    Ok(Some(u)) => u,
    _ => {
      return HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" }));
    }
  };

  // Enable email 2FA
  if let Err(e) = db::users::enable_email_twofa(&pool, user.id).await {
    return HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to enable email 2FA: {e}")
    }));
  }

  // Generate recovery codes
  let codes = crate::auth::recovery::generate_recovery_codes();
  let plaintexts: Vec<String> = codes.iter().map(|(p, _)| p.clone()).collect();
  let hashes: Vec<String> = codes.iter().map(|(_, h)| h.clone()).collect();

  let _ = db::recovery_codes::delete_by_user(&pool, user.id).await;
  if let Err(e) = db::recovery_codes::create_batch(&pool, user.id, &hashes).await {
    return HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to save recovery codes: {e}")
    }));
  }

  HttpResponse::Ok().json(TwofaEnabledResponse {
    recovery_codes: plaintexts,
  })
}

pub async fn disable_twofa(
  auth: AuthenticatedUser,
  pool: Data<PgPool>,
  body: Json<TwofaDisableRequest>,
) -> HttpResponse {
  let user = match db::users::find_by_id(&pool, auth.0.sub).await {
    Ok(Some(u)) => u,
    _ => {
      return HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" }));
    }
  };

  // Verify password
  if !verify_password(&body.password, &user.password_hash) {
    return HttpResponse::Unauthorized().json(serde_json::json!({
      "error": "Invalid password"
    }));
  }

  // Disable 2FA
  if let Err(e) = db::users::disable_twofa(&pool, user.id).await {
    return HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to disable 2FA: {e}")
    }));
  }

  // Delete recovery codes
  let _ = db::recovery_codes::delete_by_user(&pool, user.id).await;

  HttpResponse::Ok().json(serde_json::json!({ "message": "Two-factor authentication disabled" }))
}

pub async fn get_twofa_status(auth: AuthenticatedUser, pool: Data<PgPool>) -> HttpResponse {
  let user = match db::users::find_by_id(&pool, auth.0.sub).await {
    Ok(Some(u)) => u,
    _ => {
      return HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" }));
    }
  };

  let remaining = db::recovery_codes::count_unused(&pool, user.id)
    .await
    .unwrap_or(0);

  HttpResponse::Ok().json(TwofaStatusResponse {
    method: user.twofa_method.clone(),
    enabled: user.twofa_method != "none",
    recovery_codes_remaining: remaining,
  })
}

pub async fn regenerate_recovery_codes(
  auth: AuthenticatedUser,
  pool: Data<PgPool>,
  body: Json<TwofaDisableRequest>,
) -> HttpResponse {
  let user = match db::users::find_by_id(&pool, auth.0.sub).await {
    Ok(Some(u)) => u,
    _ => {
      return HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" }));
    }
  };

  if user.twofa_method == "none" {
    return HttpResponse::BadRequest().json(serde_json::json!({
      "error": "Two-factor authentication is not enabled"
    }));
  }

  // Verify password
  if !verify_password(&body.password, &user.password_hash) {
    return HttpResponse::Unauthorized().json(serde_json::json!({
      "error": "Invalid password"
    }));
  }

  // Generate new recovery codes
  let codes = crate::auth::recovery::generate_recovery_codes();
  let plaintexts: Vec<String> = codes.iter().map(|(p, _)| p.clone()).collect();
  let hashes: Vec<String> = codes.iter().map(|(_, h)| h.clone()).collect();

  let _ = db::recovery_codes::delete_by_user(&pool, user.id).await;
  if let Err(e) = db::recovery_codes::create_batch(&pool, user.id, &hashes).await {
    return HttpResponse::InternalServerError().json(serde_json::json!({
      "error": format!("Failed to save recovery codes: {e}")
    }));
  }

  HttpResponse::Ok().json(crate::models::user::RecoveryCodesResponse {
    recovery_codes: plaintexts,
  })
}
