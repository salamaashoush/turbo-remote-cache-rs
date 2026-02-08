use std::{collections::HashSet, env::VarError, fmt::Display};
use tracing::warn;

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
      other => {
        warn!(
          "Unknown storage provider '{}', falling back to Memory",
          other
        );
        StorageProvider::Memory
      }
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
pub enum SmtpTls {
  None,
  StartTls,
  Tls,
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
  pub host: String,
  pub port: u16,
  pub username: Option<String>,
  pub password: Option<String>,
  pub from: String,
  pub tls: SmtpTls,
}

#[derive(Debug, Clone)]
pub struct Config {
  pub turbo_tokens: HashSet<String>,
  pub storage_provider: StorageProvider,
  pub fs_cache_path: String,
  pub bucket_name: String,
  // Platform mode fields
  pub database_url: Option<String>,
  pub jwt_secret: String,
  pub jwt_access_expiry_secs: i64,
  pub jwt_refresh_expiry_secs: i64,
  pub legacy_tokens_enabled: bool,
  // Super admin bootstrap
  pub super_admin_email: Option<String>,
  pub super_admin_password: Option<String>,
  pub allow_registration: bool,
  // SMTP email
  pub smtp: Option<SmtpConfig>,
  pub app_url: String,
  // CORS
  pub allowed_origins: Vec<String>,
  // DB pool
  pub db_max_connections: u32,
  pub db_min_connections: u32,
  // Email verification
  pub require_email_verification: bool,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      turbo_tokens: HashSet::new(),
      storage_provider: StorageProvider::Memory,
      fs_cache_path: std::env::temp_dir()
        .to_str()
        .expect("error getting temp dir")
        .to_string(),
      bucket_name: "cache".to_string(),
      database_url: None,
      jwt_secret: "dev-secret-do-not-use-in-production".to_string(),
      jwt_access_expiry_secs: 900,
      jwt_refresh_expiry_secs: 604800,
      legacy_tokens_enabled: true,
      super_admin_email: None,
      super_admin_password: None,
      allow_registration: true,
      smtp: None,
      app_url: "http://localhost:4000".to_string(),
      allowed_origins: Vec::new(),
      db_max_connections: 10,
      db_min_connections: 2,
      require_email_verification: false,
    }
  }
}

impl Config {
  pub fn from_env() -> Result<Self, VarError> {
    let database_url = std::env::var("DATABASE_URL").ok();
    let legacy_tokens_enabled = std::env::var("LEGACY_TOKENS_ENABLED")
      .unwrap_or("true".to_string())
      .parse::<bool>()
      .unwrap_or(true);

    // Only require TURBO_TOKENS if legacy tokens are enabled
    let turbo_tokens = if legacy_tokens_enabled {
      match std::env::var("TURBO_TOKENS") {
        Ok(tokens_str) => tokens_str
          .split(',')
          .map(|s| s.trim().to_string())
          .filter(|s| !s.is_empty())
          .collect(),
        Err(_) if database_url.is_some() => HashSet::new(),
        Err(e) => return Err(e),
      }
    } else {
      HashSet::new()
    };

    Ok(Config {
      turbo_tokens,
      storage_provider: get_storage_provider(),
      fs_cache_path: get_fs_cache_path(),
      bucket_name: get_bucket_name(),
      database_url,
      jwt_secret: std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-do-not-use-in-production".to_string()),
      jwt_access_expiry_secs: std::env::var("JWT_ACCESS_EXPIRY_SECS")
        .unwrap_or("900".to_string())
        .parse()
        .unwrap_or(900),
      jwt_refresh_expiry_secs: std::env::var("JWT_REFRESH_EXPIRY_SECS")
        .unwrap_or("604800".to_string())
        .parse()
        .unwrap_or(604800),
      legacy_tokens_enabled,
      super_admin_email: std::env::var("SUPER_ADMIN_EMAIL").ok(),
      super_admin_password: std::env::var("SUPER_ADMIN_PASSWORD").ok(),
      allow_registration: std::env::var("ALLOW_REGISTRATION")
        .unwrap_or("true".to_string())
        .parse::<bool>()
        .unwrap_or(true),
      smtp: Self::smtp_from_env(),
      app_url: std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:4000".to_string()),
      allowed_origins: std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect(),
      db_max_connections: std::env::var("DB_MAX_CONNECTIONS")
        .unwrap_or("10".to_string())
        .parse()
        .unwrap_or(10),
      db_min_connections: std::env::var("DB_MIN_CONNECTIONS")
        .unwrap_or("2".to_string())
        .parse()
        .unwrap_or(2),
      require_email_verification: std::env::var("REQUIRE_EMAIL_VERIFICATION")
        .unwrap_or("false".to_string())
        .parse::<bool>()
        .unwrap_or(false),
    })
  }

  pub fn is_platform_mode(&self) -> bool {
    self.database_url.is_some()
  }

  pub fn with_turbo_tokens(mut self, turbo_tokens: Vec<String>) -> Self {
    self.turbo_tokens = turbo_tokens.into_iter().collect();
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

  fn smtp_from_env() -> Option<SmtpConfig> {
    let host = std::env::var("SMTP_HOST").ok()?;
    let port = std::env::var("SMTP_PORT")
      .unwrap_or("587".to_string())
      .parse()
      .unwrap_or(587);
    let tls = match std::env::var("SMTP_TLS")
      .unwrap_or("starttls".to_string())
      .to_lowercase()
      .as_str()
    {
      "none" => SmtpTls::None,
      "tls" => SmtpTls::Tls,
      _ => SmtpTls::StartTls,
    };
    Some(SmtpConfig {
      host,
      port,
      username: std::env::var("SMTP_USERNAME").ok(),
      password: std::env::var("SMTP_PASSWORD").ok(),
      from: std::env::var("SMTP_FROM").unwrap_or("noreply@turbo-cache.local".to_string()),
      tls,
    })
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

pub fn get_turbo_tokens() -> HashSet<String> {
  let tokens_str = std::env::var("TURBO_TOKENS").expect("TURBO_TOKENS is not set.");
  tokens_str
    .split(',')
    .map(|s| s.trim().to_string())
    .collect()
}

pub fn get_storage_provider() -> StorageProvider {
  std::env::var("STORAGE_PROVIDER")
    .unwrap_or("memory".to_string())
    .as_str()
    .into()
}
