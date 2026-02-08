use totp_rs::{Algorithm, Secret, TOTP};

/// Generate a new random base32 TOTP secret.
pub fn generate_secret() -> String {
  let secret = Secret::generate_secret();
  secret.to_encoded().to_string()
}

/// Build a TOTP instance from a base32 secret and user email.
pub fn build_totp(secret: &str, email: &str) -> Result<TOTP, String> {
  let secret_bytes = Secret::Encoded(secret.to_string())
    .to_bytes()
    .map_err(|e| format!("Invalid secret: {e}"))?;

  TOTP::new(
    Algorithm::SHA1,
    6,
    1, // skew
    30,
    secret_bytes,
    Some("TurboRemoteCache".to_string()),
    email.to_string(),
  )
  .map_err(|e| format!("Failed to build TOTP: {e}"))
}

/// Generate a base64-encoded PNG QR code for the TOTP.
pub fn generate_qr_base64(totp: &TOTP) -> Result<String, String> {
  totp
    .get_qr_base64()
    .map_err(|e| format!("Failed to generate QR code: {e}"))
}

/// Get the otpauth:// URI for manual entry.
pub fn get_otpauth_uri(totp: &TOTP) -> String {
  totp.get_url()
}

/// Verify a TOTP code against the secret.
pub fn verify_code(secret: &str, email: &str, code: &str) -> Result<bool, String> {
  let totp = build_totp(secret, email)?;
  totp
    .check_current(code)
    .map_err(|e| format!("TOTP check failed: {e}"))
}
