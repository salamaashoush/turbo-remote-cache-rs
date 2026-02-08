use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
  pub sub: Uuid,
  pub email: String,
  pub role: String,
  pub exp: i64,
  pub iat: i64,
}

pub fn create_access_token(
  user_id: Uuid,
  email: &str,
  role: &str,
  secret: &str,
  expiry_secs: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
  let now = Utc::now().timestamp();
  let claims = Claims {
    sub: user_id,
    email: email.to_string(),
    role: role.to_string(),
    exp: now + expiry_secs,
    iat: now,
  };
  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
  let token_data = decode::<Claims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::default(),
  )?;
  Ok(token_data.claims)
}
