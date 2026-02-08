/// Returns (subject, html_body)
pub fn welcome_email(name: &str, app_url: &str) -> (String, String) {
  let display = if name.is_empty() { "there" } else { name };
  let subject = "Welcome to Turbo Remote Cache".to_string();
  let html = format!(
    r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; color: #333;">
  <h2 style="color: #111;">Welcome, {display}!</h2>
  <p>Your account has been created on Turbo Remote Cache.</p>
  <p>You can now create API tokens and start caching your Turborepo builds.</p>
  <p><a href="{app_url}" style="display: inline-block; padding: 10px 20px; background: #111; color: #fff; text-decoration: none; border-radius: 6px;">Go to Dashboard</a></p>
  <p style="color: #666; font-size: 14px;">If you didn't create this account, you can safely ignore this email.</p>
</body>
</html>"#
  );
  (subject, html)
}

/// Returns (subject, html_body)
pub fn email_verification(name: &str, verify_url: &str) -> (String, String) {
  let display = if name.is_empty() { "there" } else { name };
  let subject = "Verify your email address".to_string();
  let html = format!(
    r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; color: #333;">
  <h2 style="color: #111;">Verify your email</h2>
  <p>Hi {display}, please verify your email address by clicking the button below.</p>
  <p><a href="{verify_url}" style="display: inline-block; padding: 10px 20px; background: #111; color: #fff; text-decoration: none; border-radius: 6px;">Verify Email</a></p>
  <p style="color: #666; font-size: 14px;">This link expires in 24 hours. If you didn't create an account, you can safely ignore this email.</p>
</body>
</html>"#
  );
  (subject, html)
}

/// Returns (subject, html_body)
pub fn org_invitation(inviter_name: &str, org_name: &str, app_url: &str) -> (String, String) {
  let subject = format!("You've been invited to {org_name}");
  let html = format!(
    r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; color: #333;">
  <h2 style="color: #111;">You've been invited!</h2>
  <p>{inviter_name} has added you to the organization <strong>{org_name}</strong> on Turbo Remote Cache.</p>
  <p><a href="{app_url}" style="display: inline-block; padding: 10px 20px; background: #111; color: #fff; text-decoration: none; border-radius: 6px;">Go to Dashboard</a></p>
</body>
</html>"#
  );
  (subject, html)
}

/// Returns (subject, html_body) for 2FA email OTP
pub fn twofa_email_otp(name: &str, otp: &str) -> (String, String) {
  let display = if name.is_empty() { "there" } else { name };
  let subject = "Your login verification code".to_string();
  let html = format!(
    r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; color: #333;">
  <h2 style="color: #111;">Verification Code</h2>
  <p>Hi {display}, use the following code to complete your sign-in:</p>
  <div style="text-align: center; margin: 30px 0;">
    <span style="font-size: 36px; font-weight: bold; letter-spacing: 8px; background: #f4f4f5; padding: 16px 32px; border-radius: 8px; display: inline-block;">{otp}</span>
  </div>
  <p style="color: #666; font-size: 14px;">This code expires in 10 minutes. If you didn't try to sign in, you can safely ignore this email.</p>
</body>
</html>"#
  );
  (subject, html)
}

/// Returns (subject, html_body)
pub fn password_reset(name: &str, reset_url: &str) -> (String, String) {
  let display = if name.is_empty() { "there" } else { name };
  let subject = "Reset your password".to_string();
  let html = format!(
    r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; color: #333;">
  <h2 style="color: #111;">Reset your password</h2>
  <p>Hi {display}, we received a request to reset your password.</p>
  <p><a href="{reset_url}" style="display: inline-block; padding: 10px 20px; background: #111; color: #fff; text-decoration: none; border-radius: 6px;">Reset Password</a></p>
  <p style="color: #666; font-size: 14px;">This link expires in 1 hour. If you didn't request a password reset, you can safely ignore this email.</p>
</body>
</html>"#
  );
  (subject, html)
}
