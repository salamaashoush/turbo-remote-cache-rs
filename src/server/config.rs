use std::env::VarError;

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
        .unwrap()
}

pub fn get_turbo_tokens() -> Vec<String> {
    let tokens_str = std::env::var("TURBO_TOKENS").expect("TURBO_TOKENS is not set.");
    tokens_str
        .to_string()
        .split(',')
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

pub fn get_storage_provider() -> String {
    std::env::var("STORAGE_PROVIDER").unwrap_or("file".to_string())
}
