use async_trait::async_trait;

use crate::ShippingError;
use crate::domain::entities::ShippingZone;
use crate::domain::value_objects::ShippingZoneId;
use identity::StoreId;

#[async_trait]
pub trait ShippingZoneRepository: Send + Sync {
    async fn save(&self, zone: &ShippingZone) -> Result<(), ShippingError>;
    async fn find_by_id(&self, id: ShippingZoneId) -> Result<Option<ShippingZone>, ShippingError>;
    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<ShippingZone>, ShippingError>;
    async fn find_active_by_store(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<ShippingZone>, ShippingError>;
    async fn update(&self, zone: &ShippingZone) -> Result<(), ShippingError>;
    async fn delete(&self, id: ShippingZoneId) -> Result<(), ShippingError>;
}
