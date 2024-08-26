use crate::config::{Config, StorageProvider};
use actix_web::web::Bytes;
use log::debug;
use object_store::PutPayload;
use object_store::{
  aws::AmazonS3Builder, azure::MicrosoftAzureBuilder, gcp::GoogleCloudStorageBuilder,
  local::LocalFileSystem, memory::InMemory, path::Path, Error, ObjectStore,
};
use std::{fs::create_dir_all, sync::Arc};

pub struct StorageStore {
  object_store: Arc<dyn ObjectStore>,
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

fn get_file_store(bucket_name: &str, fs_cache_path: &str) -> Result<Arc<dyn ObjectStore>, String> {
  let cache_path = format!("{}/{}", fs_cache_path, bucket_name);
  // create the folder if it doesn't exist
  create_dir_all(&cache_path).expect("error creating cache folder");
  let local = LocalFileSystem::new_with_prefix(cache_path).expect("error creating local");
  Ok(Arc::new(local))
}

fn get_memory_store() -> Result<Arc<dyn ObjectStore>, String> {
  Ok(Arc::new(InMemory::new()))
}

fn get_object_store(config: &Config) -> Result<Arc<dyn ObjectStore>, String> {
  let bucket_name = config.bucket_name.as_str();
  match config.storage_provider {
    StorageProvider::Memory => get_memory_store(),
    StorageProvider::S3 => get_s3_store(bucket_name),
    StorageProvider::Azure => get_azure_store(bucket_name),
    StorageProvider::Gcs => get_gcs_store(bucket_name),
    StorageProvider::File => get_file_store(bucket_name, &config.fs_cache_path),
  }
}

impl Default for StorageStore {
  fn default() -> Self {
    Self::new(&Config::default())
  }
}
impl StorageStore {
  pub fn new(config: &Config) -> Self {
    // create an ObjectStore
    let object_store: Arc<dyn ObjectStore> = match get_object_store(config) {
      Ok(store) => store,
      Err(e) => panic!("{}", e),
    };

    debug!("Using storage provider: {:?}", object_store);
    StorageStore { object_store }
  }

  pub async fn put(&self, path: &str, data: Bytes) -> Result<(), Error> {
    let payload = PutPayload::from(data);
    match self.object_store.put(&Path::from(path), payload).await {
      Ok(_) => Ok(()),
      Err(e) => Err(e),
    }
  }

  pub async fn get(&self, path: &str) -> Result<Bytes, Error> {
    self
      .object_store
      .get(&Path::from(path))
      .await
      .expect("Failed to get artifact.")
      .bytes()
      .await
  }

  pub async fn exists(&self, path: &str) -> bool {
    self.object_store.head(&Path::from(path)).await.is_ok()
  }
}
