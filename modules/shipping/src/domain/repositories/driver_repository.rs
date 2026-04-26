use async_trait::async_trait;

use crate::ShippingError;
use crate::domain::entities::Driver;
use crate::domain::value_objects::DriverId;
use identity::StoreId;

#[async_trait]
pub trait DriverRepository: Send + Sync {
    async fn save(&self, driver: &Driver) -> Result<(), ShippingError>;
    async fn find_by_id(&self, id: DriverId) -> Result<Option<Driver>, ShippingError>;
    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<Driver>, ShippingError>;
    async fn find_available_by_store(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<Driver>, ShippingError>;
    async fn update(&self, driver: &Driver) -> Result<(), ShippingError>;
    async fn delete(&self, id: DriverId) -> Result<(), ShippingError>;
}
