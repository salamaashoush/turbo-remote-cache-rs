use crate::config::{Config, StorageProvider};
use actix_web::web::Bytes;
use futures_util::StreamExt;
use futures_util::stream::BoxStream;
use object_store::PutPayload;
use object_store::{
  Error, ObjectStore, ObjectStoreExt, aws::AmazonS3Builder, azure::MicrosoftAzureBuilder,
  gcp::GoogleCloudStorageBuilder, local::LocalFileSystem, memory::InMemory, path::Path,
};
use std::{fs::create_dir_all, sync::Arc};
use tracing::debug;

pub struct StorageStore {
  object_store: Arc<dyn ObjectStore>,
}

fn get_gcs_store(bucket_name: &str) -> Result<Arc<dyn ObjectStore>, String> {
  let gcs = GoogleCloudStorageBuilder::from_env()
    .with_bucket_name(bucket_name)
    .build()
    .map_err(|e| format!("error creating gcs: {e}"))?;
  Ok(Arc::new(gcs))
}

fn get_azure_store(bucket_name: &str) -> Result<Arc<dyn ObjectStore>, String> {
  let azure = MicrosoftAzureBuilder::from_env()
    .with_container_name(bucket_name)
    .build()
    .map_err(|e| format!("error creating azure: {e}"))?;
  Ok(Arc::new(azure))
}

fn get_s3_store(bucket_name: &str) -> Result<Arc<dyn ObjectStore>, String> {
  let s3 = AmazonS3Builder::from_env()
    .with_bucket_name(bucket_name)
    .build()
    .map_err(|e| format!("error creating s3: {e}"))?;
  Ok(Arc::new(s3))
}

fn get_file_store(bucket_name: &str, fs_cache_path: &str) -> Result<Arc<dyn ObjectStore>, String> {
  let cache_path = format!("{}/{}", fs_cache_path, bucket_name);
  create_dir_all(&cache_path).map_err(|e| format!("error creating cache folder: {e}"))?;
  let local = LocalFileSystem::new_with_prefix(cache_path)
    .map_err(|e| format!("error creating local store: {e}"))?;
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
    Self::new(&Config::default()).expect("Failed to create default StorageStore")
  }
}
impl StorageStore {
  pub fn new(config: &Config) -> Result<Self, String> {
    let object_store = get_object_store(config)?;
    debug!("Using storage provider: {:?}", object_store);
    Ok(StorageStore { object_store })
  }

  pub async fn put(&self, path: &str, data: Bytes) -> Result<(), Error> {
    let payload = PutPayload::from(data);
    self.object_store.put(&Path::from(path), payload).await?;
    Ok(())
  }

  pub async fn get(&self, path: &str) -> Result<Bytes, Error> {
    self
      .object_store
      .get(&Path::from(path))
      .await?
      .bytes()
      .await
  }

  pub fn get_stream(&self, path: &str) -> BoxStream<'static, Result<Bytes, Error>> {
    let store = self.object_store.clone();
    let path = Path::from(path);
    Box::pin(
      futures_util::stream::once(async move { store.get(&path).await }).flat_map(|result| {
        match result {
          Ok(get_result) => get_result.into_stream(),
          Err(e) => Box::pin(futures_util::stream::once(async move { Err(e) })),
        }
      }),
    )
  }

  pub async fn exists(&self, path: &str) -> bool {
    self.object_store.head(&Path::from(path)).await.is_ok()
  }

  pub async fn put_tag(&self, path: &str, tag: &str) -> Result<(), Error> {
    let tag_path = format!("{}.tag", path);
    let payload = PutPayload::from(Bytes::from(tag.to_string()));
    self
      .object_store
      .put(&Path::from(tag_path), payload)
      .await?;
    Ok(())
  }

  pub async fn get_tag(&self, path: &str) -> Option<String> {
    let tag_path = format!("{}.tag", path);
    match self.object_store.get(&Path::from(tag_path)).await {
      Ok(result) => match result.bytes().await {
        Ok(bytes) => String::from_utf8(bytes.to_vec()).ok(),
        Err(_) => None,
      },
      Err(_) => None,
    }
  }

  pub async fn put_duration(&self, path: &str, duration_ms: i32) -> Result<(), Error> {
    let dur_path = format!("{}.duration", path);
    let payload = PutPayload::from(Bytes::from(duration_ms.to_string()));
    self
      .object_store
      .put(&Path::from(dur_path), payload)
      .await?;
    Ok(())
  }

  pub async fn get_duration(&self, path: &str) -> Option<i32> {
    let dur_path = format!("{}.duration", path);
    match self.object_store.get(&Path::from(dur_path)).await {
      Ok(result) => match result.bytes().await {
        Ok(bytes) => String::from_utf8(bytes.to_vec())
          .ok()
          .and_then(|s| s.parse().ok()),
        Err(_) => None,
      },
      Err(_) => None,
    }
  }

  pub async fn delete(&self, path: &str) -> Result<(), Error> {
    self.object_store.delete(&Path::from(path)).await?;
    // Also try to delete the tag and duration sidecars
    let tag_path = format!("{}.tag", path);
    let dur_path = format!("{}.duration", path);
    let _ = self.object_store.delete(&Path::from(tag_path)).await;
    let _ = self.object_store.delete(&Path::from(dur_path)).await;
    Ok(())
  }

  pub async fn delete_prefix(&self, prefix: &str) -> Result<u64, Error> {
    let prefix_path = Path::from(prefix);
    let mut count = 0u64;

    let mut list_stream = self.object_store.list(Some(&prefix_path));
    while let Some(meta) = list_stream.next().await {
      if let Ok(meta) = meta {
        let _ = self.object_store.delete(&meta.location).await;
        count += 1;
      }
    }

    Ok(count)
  }
}
