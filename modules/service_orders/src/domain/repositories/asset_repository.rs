use async_trait::async_trait;
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::domain::entities::Asset;
use crate::domain::value_objects::{AssetId, AssetType};

#[async_trait]
pub trait AssetRepository: Send + Sync {
    async fn save(&self, asset: &Asset) -> Result<(), ServiceOrdersError>;
    async fn update(&self, asset: &Asset) -> Result<(), ServiceOrdersError>;
    async fn find_by_id(&self, id: AssetId) -> Result<Option<Asset>, ServiceOrdersError>;
    async fn list_by_store(
        &self,
        store_id: Uuid,
        only_active: bool,
        asset_type_filter: Option<AssetType>,
    ) -> Result<Vec<Asset>, ServiceOrdersError>;
    async fn list_by_customer(&self, customer_id: Uuid) -> Result<Vec<Asset>, ServiceOrdersError>;
}
