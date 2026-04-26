use async_trait::async_trait;

use crate::CatalogError;
use crate::domain::entities::ImageStorageProvider;
use crate::domain::value_objects::ImageStorageProviderId;
use identity::StoreId;

#[async_trait]
pub trait ImageStorageProviderRepository: Send + Sync {
    async fn save(&self, provider: &ImageStorageProvider) -> Result<(), CatalogError>;
    async fn find_by_id(
        &self,
        id: ImageStorageProviderId,
    ) -> Result<Option<ImageStorageProvider>, CatalogError>;
    async fn find_by_store(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<ImageStorageProvider>, CatalogError>;
    async fn find_default(
        &self,
        store_id: StoreId,
    ) -> Result<Option<ImageStorageProvider>, CatalogError>;
    async fn update(&self, provider: &ImageStorageProvider) -> Result<(), CatalogError>;
    async fn delete(&self, id: ImageStorageProviderId) -> Result<(), CatalogError>;
    async fn unset_default_except(
        &self,
        store_id: StoreId,
        keep: ImageStorageProviderId,
    ) -> Result<(), CatalogError>;
}
