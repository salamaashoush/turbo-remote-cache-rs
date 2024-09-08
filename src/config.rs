use std::{env::VarError, fmt::Display};

#[derive(Debug, Clone, Default)]
pub enum StorageProvider {
  S3,
  File,
  Gcs,
  Azure,
  #[default]
  Memory,
}

impl From<&str> for StorageProvider {
  fn from(s: &str) -> Self {
    match s {
      "s3" => StorageProvider::S3,
      "file" => StorageProvider::File,
      "gcs" => StorageProvider::Gcs,
      "azure" => StorageProvider::Azure,
      "memory" => StorageProvider::Memory,
      _ => panic!("Invalid storage provider"),
    }
  }
}

impl Display for StorageProvider {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      StorageProvider::S3 => write!(f, "S3"),
      StorageProvider::File => write!(f, "File"),
      StorageProvider::Gcs => write!(f, "GCS"),
      StorageProvider::Azure => write!(f, "Azure"),
      StorageProvider::Memory => write!(f, "Memory"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Config {
  pub turbo_tokens: Vec<String>,
  pub storage_provider: StorageProvider,
  pub fs_cache_path: String,
  pub bucket_name: String,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      turbo_tokens: vec![],
      storage_provider: StorageProvider::Memory,
      fs_cache_path: std::env::temp_dir()
        .to_str()
        .expect("error getting temp dir")
        .to_string(),
      bucket_name: "cache".to_string(),
    }
  }
}

impl Config {
  pub fn from_env() -> Result<Self, VarError> {
    Ok(Config {
      turbo_tokens: get_turbo_tokens(),
      storage_provider: get_storage_provider(),
      fs_cache_path: get_fs_cache_path(),
      bucket_name: get_bucket_name(),
    })
  }

  pub fn with_turbo_tokens(mut self, turbo_tokens: Vec<String>) -> Self {
    self.turbo_tokens = turbo_tokens;
    self
  }

  pub fn with_storage_provider(mut self, storage_provider: StorageProvider) -> Self {
    self.storage_provider = storage_provider;
    self
  }

  pub fn with_fs_cache_path(mut self, fs_cache_path: String) -> Self {
    self.fs_cache_path = fs_cache_path;
    self
  }

  pub fn with_bucket_name(mut self, bucket_name: String) -> Self {
    self.bucket_name = bucket_name;
    self
  }
}

pub fn get_fs_cache_path() -> String {
  std::env::var("FS_PATH")
    .or_else(|_| match std::env::temp_dir().to_str() {
      Some(dir) => Ok(dir.to_string()),
      None => Err(VarError::NotPresent),
    })
    .unwrap()
}

pub fn get_bucket_name() -> String {
  std::env::var("BUCKET_NAME").unwrap_or("cache".to_string())
}

pub fn get_port() -> u16 {
  std::env::var("PORT")
    .unwrap_or("4000".to_string())
    .parse()
    .expect("PORT must be a number")
}

pub fn get_turbo_tokens() -> Vec<String> {
  let tokens_str = std::env::var("TURBO_TOKENS").expect("TURBO_TOKENS is not set.");
  tokens_str.split(',').map(|s| s.to_string()).collect()
}

pub fn get_storage_provider() -> StorageProvider {
  std::env::var("STORAGE_PROVIDER")
    .unwrap_or("memory".to_string())
    .as_str()
    .into()
}
