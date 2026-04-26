//! Google Cloud Storage adapter — STUB.

use async_trait::async_trait;

use super::storage_adapter::{ImageStorageAdapter, UploadRequest, UploadResult};
use crate::CatalogError;

#[derive(Debug, Default, Clone)]
pub struct GcsAdapter;

impl GcsAdapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str = "GCS adapter is not yet wired. Configure credentials and implement the Google Cloud Storage API.";

#[async_trait]
impl ImageStorageAdapter for GcsAdapter {
    async fn upload(&self, _req: UploadRequest) -> Result<UploadResult, CatalogError> {
        Err(CatalogError::StorageProviderError(NOT_WIRED.to_string()))
    }
    async fn delete(&self, _storage_key: &str) -> Result<(), CatalogError> {
        Err(CatalogError::StorageProviderError(NOT_WIRED.to_string()))
    }
}
