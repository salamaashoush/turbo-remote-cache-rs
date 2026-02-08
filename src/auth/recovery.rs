use sha2::{Digest, Sha256};

/// Generate 10 recovery codes in the format `XXXX-XXXX`.
/// Returns Vec of (plaintext, sha256_hash).
pub fn generate_recovery_codes() -> Vec<(String, String)> {
  (0..10)
    .map(|_| {
      let bytes: [u8; 4] = rand::random();
      let hex_str = hex::encode(bytes); // 8 hex chars
      let code = format!("{}-{}", &hex_str[..4], &hex_str[4..]);
      let hash = sha256_hash(&code);
      (code, hash)
    })
    .collect()
}

pub fn sha256_hash(input: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(input.as_bytes());
  hex::encode(hasher.finalize())
}
