use std::sync::Arc;

use actix_web::{
  HttpResponse,
  web::{Data, Form, ServiceConfig, get, post, scope},
};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::warn;
use uuid::Uuid;

use crate::auth::{api_token, password};
use crate::db;

// ---------------------------------------------------------------------------
// Forms
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct LoginPageQuery {
  redirect_uri: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
  email: String,
  password: String,
  redirect_uri: String,
}

#[derive(Deserialize)]
pub struct TwofaForm {
  user_id: String,
  code: String,
  redirect_uri: String,
}

// ---------------------------------------------------------------------------
// Shared HTML helpers
// ---------------------------------------------------------------------------

fn html_page(title: &str, body: &str) -> HttpResponse {
  let html = format!(
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title}</title>
<style>
  *{{box-sizing:border-box;margin:0;padding:0}}
  body{{font-family:system-ui,-apple-system,sans-serif;background:#0a0a0a;color:#ededed;display:flex;align-items:center;justify-content:center;min-height:100vh}}
  .card{{background:#1a1a1a;border:1px solid #333;border-radius:12px;padding:2rem;width:100%;max-width:400px}}
  h1{{font-size:1.25rem;margin-bottom:1.5rem;text-align:center}}
  label{{display:block;font-size:.875rem;color:#aaa;margin-bottom:.25rem}}
  input[type=email],input[type=password],input[type=text]{{width:100%;padding:.625rem .75rem;background:#0a0a0a;border:1px solid #444;border-radius:8px;color:#ededed;font-size:.9rem;margin-bottom:1rem}}
  input:focus{{outline:none;border-color:#3b82f6}}
  button{{width:100%;padding:.625rem;background:#3b82f6;color:#fff;border:none;border-radius:8px;font-size:.9rem;cursor:pointer;font-weight:600}}
  button:hover{{background:#2563eb}}
  .error{{background:#451a1a;border:1px solid #7f1d1d;color:#fca5a5;padding:.75rem;border-radius:8px;margin-bottom:1rem;font-size:.875rem}}
  .subtitle{{font-size:.8rem;color:#888;text-align:center;margin-bottom:1.5rem}}
</style>
</head>
<body>
<div class="card">
{body}
</div>
</body>
</html>"#,
    title = title,
    body = body,
  );
  HttpResponse::Ok().content_type("text/html").body(html)
}

fn error_html(title: &str, message: &str, back_uri: Option<&str>) -> HttpResponse {
  let back_link = back_uri
    .map(|uri| format!(r#"<p style="text-align:center;margin-top:1rem"><a href="{uri}" style="color:#3b82f6">Try again</a></p>"#))
    .unwrap_or_default();
  html_page(
    title,
    &format!(r#"<h1>{title}</h1><div class="error">{message}</div>{back_link}"#),
  )
}

// ---------------------------------------------------------------------------
// GET /turborepo/token — render login form
// ---------------------------------------------------------------------------

pub async fn login_page(
  query: actix_web::web::Query<LoginPageQuery>,
) -> HttpResponse {
  let redirect_uri = html_escape(&query.redirect_uri);
  html_page(
    "Turbo Remote Cache — Sign In",
    &format!(
      r#"<h1>Turbo Remote Cache</h1>
<p class="subtitle">Sign in to authorize the Turbo CLI</p>
<form method="post" action="/turborepo/token">
  <input type="hidden" name="redirect_uri" value="{redirect_uri}">
  <label for="email">Email</label>
  <input type="email" id="email" name="email" required autofocus>
  <label for="password">Password</label>
  <input type="password" id="password" name="password" required>
  <button type="submit">Sign in</button>
</form>"#
    ),
  )
}

// ---------------------------------------------------------------------------
// POST /turborepo/token — authenticate and issue token
// ---------------------------------------------------------------------------

pub async fn login_submit(
  pool: Data<PgPool>,
  config: Data<Arc<crate::config::Config>>,
  form: Form<LoginForm>,
) -> HttpResponse {
  let redirect_uri = &form.redirect_uri;

  // Look up user
  let user = match db::users::find_by_email(&pool, &form.email).await {
    Ok(Some(u)) => u,
    _ => return error_html("Login Failed", "Invalid email or password.", Some(&format!("/turborepo/token?redirect_uri={}", urlencoding::encode(redirect_uri)))),
  };

  // Verify password
  if !password::verify_password(&form.password, &user.password_hash) {
    warn!("Failed login attempt for email={}", form.email);
    return error_html("Login Failed", "Invalid email or password.", Some(&format!("/turborepo/token?redirect_uri={}", urlencoding::encode(redirect_uri))));
  }

  // Check active
  if !user.is_active {
    warn!("Login attempt on deactivated account email={}", form.email);
    return error_html("Account Disabled", "Your account has been deactivated. Contact an administrator.", None);
  }

  // Check email verification if required
  if config.require_email_verification && !user.email_verified {
    return error_html("Email Not Verified", "Please verify your email before logging in.", None);
  }

  // Check 2FA
  if user.twofa_method != "none" {
    let escaped_redirect = html_escape(redirect_uri);
    return html_page(
      "Two-Factor Authentication",
      &format!(
        r#"<h1>Two-Factor Authentication</h1>
<p class="subtitle">Enter your {} code{}</p>
<form method="post" action="/turborepo/token/verify-2fa">
  <input type="hidden" name="redirect_uri" value="{escaped_redirect}">
  <input type="hidden" name="user_id" value="{}">
  <label for="code">Verification code</label>
  <input type="text" id="code" name="code" required autofocus autocomplete="one-time-code" inputmode="numeric" pattern="[0-9a-fA-F\-]{{4,}}">
  <button type="submit">Verify</button>
</form>"#,
        match user.twofa_method.as_str() {
          "totp" => "authenticator app",
          "email" => "email",
          _ => "verification",
        },
        if user.twofa_method == "totp" { " or a recovery code" } else { "" },
        user.id,
      ),
    );
  }

  // No 2FA — issue token directly
  create_token_and_redirect(&pool, user.id, redirect_uri).await
}

// ---------------------------------------------------------------------------
// POST /turborepo/token/verify-2fa — verify 2FA code and issue token
// ---------------------------------------------------------------------------

pub async fn verify_2fa_submit(
  pool: Data<PgPool>,
  form: Form<TwofaForm>,
) -> HttpResponse {
  let redirect_uri = &form.redirect_uri;

  let user_id = match form.user_id.parse::<Uuid>() {
    Ok(id) => id,
    Err(_) => return error_html("Error", "Invalid session.", None),
  };

  let user = match db::users::find_by_id(&pool, user_id).await {
    Ok(Some(u)) => u,
    _ => return error_html("Error", "User not found.", None),
  };

  let code = form.code.trim();

  // Try TOTP first
  let code_valid = match user.twofa_method.as_str() {
    "totp" => match &user.totp_secret {
      Some(secret) => {
        crate::auth::totp::verify_code(secret, &user.email, code).unwrap_or(false)
      }
      None => false,
    },
    _ => false,
  };

  // Fallback: recovery code
  let used_recovery = if !code_valid {
    let recovery_hash = crate::auth::recovery::sha256_hash(code);
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
    return error_html(
      "Verification Failed",
      "Invalid verification code. Please try again.",
      Some(&format!(
        "/turborepo/token?redirect_uri={}",
        urlencoding::encode(redirect_uri)
      )),
    );
  }

  create_token_and_redirect(&pool, user.id, redirect_uri).await
}

// ---------------------------------------------------------------------------
// Token creation helper
// ---------------------------------------------------------------------------

async fn create_token_and_redirect(pool: &PgPool, user_id: Uuid, redirect_uri: &str) -> HttpResponse {
  // Find user's first org
  let orgs = match db::organizations::list_by_user(pool, user_id).await {
    Ok(o) => o,
    Err(e) => {
      warn!("Failed to list orgs for user {}: {}", user_id, e);
      return error_html("Error", "Failed to look up your organizations.", None);
    }
  };

  let org = match orgs.first() {
    Some(o) => o,
    None => return error_html("No Organization", "You are not a member of any organization. Ask an admin to add you.", None),
  };

  // Generate token
  let raw_token = api_token::generate_token();
  let hash = api_token::hash_token(&raw_token);
  let prefix = api_token::token_prefix(&raw_token);
  let scopes = vec!["cache:read".to_string(), "cache:write".to_string()];

  if let Err(e) = db::api_tokens::create(
    pool,
    org.id,
    None,
    "Turbo CLI",
    &hash,
    &prefix,
    &scopes,
    user_id,
    None,
  )
  .await
  {
    warn!("Failed to create API token: {}", e);
    return error_html("Error", "Failed to create API token.", None);
  }

  // Redirect back to CLI with the token
  let location = format!(
    "{}{}token={}",
    redirect_uri,
    if redirect_uri.contains('?') { "&" } else { "?" },
    urlencoding::encode(&raw_token),
  );

  HttpResponse::Found()
    .insert_header(("Location", location))
    .finish()
}

// ---------------------------------------------------------------------------
// Minimal HTML escaping
// ---------------------------------------------------------------------------

fn html_escape(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}

// ---------------------------------------------------------------------------
// Route configuration
// ---------------------------------------------------------------------------

pub fn configure(cfg: &mut ServiceConfig) {
  cfg.service(
    scope("/turborepo")
      .route("/token", get().to(login_page))
      .route("/token", post().to(login_submit))
      .route("/token/verify-2fa", post().to(verify_2fa_submit)),
  );
}
