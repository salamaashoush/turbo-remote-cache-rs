use argon2::{
  Argon2,
  password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  let hash = argon2.hash_password(password.as_bytes(), &salt)?;
  Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
  let parsed_hash = match PasswordHash::new(hash) {
    Ok(h) => h,
    Err(_) => return false,
  };
  Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok()
}
