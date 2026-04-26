use async_trait::async_trait;

use crate::ShippingError;
use crate::domain::entities::ShippingMethod;
use crate::domain::value_objects::ShippingMethodId;
use identity::StoreId;

#[async_trait]
pub trait ShippingMethodRepository: Send + Sync {
    async fn save(&self, method: &ShippingMethod) -> Result<(), ShippingError>;
    async fn find_by_id(
        &self,
        id: ShippingMethodId,
    ) -> Result<Option<ShippingMethod>, ShippingError>;
    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<ShippingMethod>, ShippingError>;
    async fn find_active_by_store(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<ShippingMethod>, ShippingError>;
    async fn update(&self, method: &ShippingMethod) -> Result<(), ShippingError>;
    async fn delete(&self, id: ShippingMethodId) -> Result<(), ShippingError>;
}
