//! Cloudinary adapter — STUB.

use async_trait::async_trait;

use super::storage_adapter::{ImageStorageAdapter, UploadRequest, UploadResult};
use crate::CatalogError;

#[derive(Debug, Default, Clone)]
pub struct CloudinaryAdapter;

impl CloudinaryAdapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str = "Cloudinary adapter is not yet wired. Configure cloud_name/api_key/api_secret and implement upload/delete.";

#[async_trait]
impl ImageStorageAdapter for CloudinaryAdapter {
    async fn upload(&self, _req: UploadRequest) -> Result<UploadResult, CatalogError> {
        Err(CatalogError::StorageProviderError(NOT_WIRED.to_string()))
    }
    async fn delete(&self, _storage_key: &str) -> Result<(), CatalogError> {
        Err(CatalogError::StorageProviderError(NOT_WIRED.to_string()))
    }
}
