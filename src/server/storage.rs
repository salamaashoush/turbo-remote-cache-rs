use actix_web::web::Bytes;
use log::debug;
use object_store::{
    aws::AmazonS3Builder, azure::MicrosoftAzureBuilder, gcp::GoogleCloudStorageBuilder,
    local::LocalFileSystem, path::Path, Error, ObjectStore,
};

use std::{fs::create_dir_all, sync::Arc};

use crate::server::config::{get_bucket_name, get_fs_cache_path, get_storage_provider};
pub struct StorageStore {
    object_store: Arc<dyn ObjectStore>,
}
enum Provider {
    S3,
    FILE,
    GCS,
    AZURE,
}

fn detect_provider_from_env() -> Provider {
    let provider = get_storage_provider();
    match provider.as_str() {
        "s3" => Provider::S3,
        "file" => Provider::FILE,
        "gcs" => Provider::GCS,
        "azure" => Provider::AZURE,
        _ => panic!("Invalid storage provider"),
    }
}

fn get_gcs_store(bucket_name: &str) -> Result<Arc<dyn ObjectStore>, String> {
    let gcs = GoogleCloudStorageBuilder::from_env()
        .with_bucket_name(bucket_name)
        .build()
        .expect("error creating gcs");
    Ok(Arc::new(gcs))
}

fn get_azure_store(bucket_name: &str) -> Result<Arc<dyn ObjectStore>, String> {
    let azure = MicrosoftAzureBuilder::from_env()
        .with_container_name(bucket_name)
        .build()
        .expect("error creating azure");

    Ok(Arc::new(azure))
}

fn get_s3_store(bucket_name: &str) -> Result<Arc<dyn ObjectStore>, String> {
    let s3 = AmazonS3Builder::from_env()
        .with_bucket_name(bucket_name)
        .build()
        .expect("error creating s3");

    Ok(Arc::new(s3))
}

fn get_file_store(bucket_name: &str) -> Result<Arc<dyn ObjectStore>, String> {
    let fs_root = get_fs_cache_path();
    let cache_path = format!("{}/{}", fs_root, bucket_name);
    // create the folder if it doesn't exist
    create_dir_all(&cache_path).expect("error creating cache folder");
    let local = LocalFileSystem::new_with_prefix(cache_path).expect("error creating local");
    Ok(Arc::new(local))
}

fn get_object_store(provider: Provider) -> Result<Arc<dyn ObjectStore>, String> {
    let bucket_name = get_bucket_name();
    match provider {
        Provider::S3 => get_s3_store(&bucket_name),
        Provider::AZURE => get_azure_store(&bucket_name),
        Provider::GCS => get_gcs_store(&bucket_name),
        Provider::FILE => get_file_store(&bucket_name),
    }
}

impl StorageStore {
    pub fn new() -> Self {
        let provider = detect_provider_from_env();
        // create an ObjectStore
        let object_store: Arc<dyn ObjectStore> = match get_object_store(provider) {
            Ok(store) => store,
            Err(e) => panic!("{}", e),
        };

        debug!("Using storage provider: {:?}", object_store);
        StorageStore { object_store }
    }

    pub async fn put(&self, path: &str, data: Bytes) {
        self.object_store
            .put(&Path::from(path), data)
            .await
            .expect("Failed to put artifact.")
    }

    pub async fn get(&self, path: &str) -> Result<Bytes, Error> {
        self.object_store
            .get(&Path::from(path))
            .await
            .expect("Failed to get artifact.")
            .bytes()
            .await
    }

    pub async fn exists(&self, path: &str) -> bool {
        match self.object_store.head(&Path::from(path)).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
