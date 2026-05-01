use async_trait::async_trait;

use crate::ShippingError;
use crate::domain::entities::DeliveryProvider;
use crate::domain::value_objects::DeliveryProviderId;
use identity::StoreId;

#[async_trait]
pub trait DeliveryProviderRepository: Send + Sync {
    async fn save(&self, provider: &DeliveryProvider) -> Result<(), ShippingError>;
    async fn find_by_id(
        &self,
        id: DeliveryProviderId,
    ) -> Result<Option<DeliveryProvider>, ShippingError>;
    async fn find_by_store(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<DeliveryProvider>, ShippingError>;
    async fn find_default(
        &self,
        store_id: StoreId,
    ) -> Result<Option<DeliveryProvider>, ShippingError>;
    async fn update(&self, provider: &DeliveryProvider) -> Result<(), ShippingError>;
    async fn delete(&self, id: DeliveryProviderId) -> Result<(), ShippingError>;
    /// Clear `is_default=true` from every provider of `store_id` other than `keep`.
    async fn unset_default_except(
        &self,
        store_id: StoreId,
        keep: DeliveryProviderId,
    ) -> Result<(), ShippingError>;
}
