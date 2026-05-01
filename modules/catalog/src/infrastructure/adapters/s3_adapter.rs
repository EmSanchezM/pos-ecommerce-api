//! AWS S3 storage adapter — STUB.

use async_trait::async_trait;

use super::storage_adapter::{ImageStorageAdapter, UploadRequest, UploadResult};
use crate::CatalogError;

#[derive(Debug, Default, Clone)]
pub struct S3Adapter;

impl S3Adapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str =
    "S3 adapter is not yet wired. Configure credentials and implement the AWS SDK calls.";

#[async_trait]
impl ImageStorageAdapter for S3Adapter {
    async fn upload(&self, _req: UploadRequest) -> Result<UploadResult, CatalogError> {
        Err(CatalogError::StorageProviderError(NOT_WIRED.to_string()))
    }
    async fn delete(&self, _storage_key: &str) -> Result<(), CatalogError> {
        Err(CatalogError::StorageProviderError(NOT_WIRED.to_string()))
    }
}
