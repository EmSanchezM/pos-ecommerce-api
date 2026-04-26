//! ImageStorageRegistry — selects an adapter for a `StorageProviderType`.

use std::str::FromStr;
use std::sync::Arc;

use crate::CatalogError;
use crate::domain::value_objects::StorageProviderType;

use super::azure_blob_adapter::AzureBlobAdapter;
use super::cloudinary_adapter::CloudinaryAdapter;
use super::gcs_adapter::GcsAdapter;
use super::local_server_adapter::LocalServerAdapter;
use super::s3_adapter::S3Adapter;
use super::storage_adapter::ImageStorageAdapter;

pub trait ImageStorageRegistry: Send + Sync {
    fn for_type(&self, provider_type: StorageProviderType) -> Arc<dyn ImageStorageAdapter>;

    fn for_type_str(
        &self,
        provider_type: &str,
    ) -> Result<Arc<dyn ImageStorageAdapter>, CatalogError> {
        let pt = StorageProviderType::from_str(provider_type)?;
        Ok(self.for_type(pt))
    }
}

pub struct DefaultImageStorageRegistry {
    local: Arc<dyn ImageStorageAdapter>,
    s3: Arc<dyn ImageStorageAdapter>,
    gcs: Arc<dyn ImageStorageAdapter>,
    cloudinary: Arc<dyn ImageStorageAdapter>,
    azure: Arc<dyn ImageStorageAdapter>,
}

impl DefaultImageStorageRegistry {
    pub fn new() -> Self {
        Self {
            local: Arc::new(LocalServerAdapter::from_env()),
            s3: Arc::new(S3Adapter::new()),
            gcs: Arc::new(GcsAdapter::new()),
            cloudinary: Arc::new(CloudinaryAdapter::new()),
            azure: Arc::new(AzureBlobAdapter::new()),
        }
    }

    /// Override the LocalServer adapter (useful when API tests want to point
    /// at a temp dir without touching env vars).
    pub fn with_local(local: Arc<dyn ImageStorageAdapter>) -> Self {
        Self {
            local,
            s3: Arc::new(S3Adapter::new()),
            gcs: Arc::new(GcsAdapter::new()),
            cloudinary: Arc::new(CloudinaryAdapter::new()),
            azure: Arc::new(AzureBlobAdapter::new()),
        }
    }
}

impl Default for DefaultImageStorageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageStorageRegistry for DefaultImageStorageRegistry {
    fn for_type(&self, provider_type: StorageProviderType) -> Arc<dyn ImageStorageAdapter> {
        match provider_type {
            StorageProviderType::LocalServer => self.local.clone(),
            StorageProviderType::S3 => self.s3.clone(),
            StorageProviderType::Gcs => self.gcs.clone(),
            StorageProviderType::Cloudinary => self.cloudinary.clone(),
            StorageProviderType::AzureBlob => self.azure.clone(),
        }
    }
}
