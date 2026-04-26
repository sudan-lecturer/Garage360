use std::sync::Arc;

use object_store::aws::AmazonS3Builder;
use object_store::path::Path;
use object_store::{ObjectStore, PutPayload};

use crate::config::AppConfig;
use crate::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct Storage {
    store: Arc<dyn ObjectStore>,
}

impl Storage {
    pub async fn new(config: &AppConfig) -> AppResult<Self> {
        let endpoint = normalize_endpoint(&config.minio_endpoint);
        let store = AmazonS3Builder::new()
            .with_bucket_name(&config.minio_bucket)
            .with_endpoint(endpoint)
            .with_access_key_id(&config.minio_access_key)
            .with_secret_access_key(&config.minio_secret_key)
            .with_allow_http(true)
            .build()
            .map_err(|err| AppError::Internal(format!("Failed to build object store: {err}")))?;

        Ok(Self {
            store: Arc::new(store),
        })
    }

    pub async fn put_bytes(&self, key: &str, bytes: Vec<u8>) -> AppResult<()> {
        self.store
            .put(&Path::from(key), PutPayload::from(bytes))
            .await
            .map_err(|err| AppError::Internal(format!("Object store put failed: {err}")))?;
        Ok(())
    }

    pub async fn get_bytes(&self, key: &str) -> AppResult<Vec<u8>> {
        let result = self
            .store
            .get(&Path::from(key))
            .await
            .map_err(|_| AppError::NotFound("Object not found".into()))?;
        let payload = result
            .bytes()
            .await
            .map_err(|err| AppError::Internal(format!("Object store read failed: {err}")))?;
        Ok(payload.to_vec())
    }

    pub async fn delete(&self, key: &str) -> AppResult<()> {
        self.store
            .delete(&Path::from(key))
            .await
            .map_err(|err| AppError::Internal(format!("Object delete failed: {err}")))?;
        Ok(())
    }
}

fn normalize_endpoint(endpoint: &str) -> String {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.to_string()
    } else {
        format!("http://{endpoint}")
    }
}
