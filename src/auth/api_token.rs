use sha2::{Digest, Sha256};

/// Generate a new API token in the format `trcs_<64 hex chars>`.
pub fn generate_token() -> String {
  let bytes: [u8; 32] = rand::random();
  format!("trcs_{}", hex::encode(bytes))
}

/// SHA-256 hash a token for storage.
pub fn hash_token(token: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());
  hex::encode(hasher.finalize())
}

/// Extract the display prefix from a token (first 12 chars).
pub fn token_prefix(token: &str) -> String {
  token.chars().take(12).collect()
}
