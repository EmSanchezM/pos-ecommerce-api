//! ImageStorageAdapter trait — every storage backend implements this.

use async_trait::async_trait;
use uuid::Uuid;

use crate::CatalogError;

/// Bytes + metadata to upload.
#[derive(Debug, Clone)]
pub struct UploadRequest {
    pub store_id: Uuid,
    pub bytes: Vec<u8>,
    pub original_filename: String,
    pub content_type: String,
}

/// What the adapter returns after uploading.
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// Public URL the frontend renders.
    pub url: String,
    /// Adapter-specific identifier needed to delete the file later.
    pub storage_key: String,
    pub content_type: String,
    pub size_bytes: i64,
}

#[async_trait]
pub trait ImageStorageAdapter: Send + Sync {
    async fn upload(&self, req: UploadRequest) -> Result<UploadResult, CatalogError>;
    async fn delete(&self, storage_key: &str) -> Result<(), CatalogError>;
}
