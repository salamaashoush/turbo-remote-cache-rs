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
    std::env::var("BUCKET_NAME").expect("BUCKET_NAME is not set.")
}

pub fn get_port() -> u16 {
    std::env::var("PORT")
        .unwrap_or_else(|_| "4000".to_string())
        .parse()
        .unwrap()
}

pub fn get_turbo_token() -> String {
    std::env::var("TURBO_TOKEN").expect("TURBO_TOKEN is not set.")
}

pub fn get_storage_provider() -> String {
    std::env::var("STORAGE_PROVIDER").unwrap_or_else(|_| "file".to_string())
}
