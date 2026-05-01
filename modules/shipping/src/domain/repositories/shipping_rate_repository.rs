use async_trait::async_trait;

use crate::ShippingError;
use crate::domain::entities::ShippingRate;
use crate::domain::value_objects::{ShippingMethodId, ShippingRateId, ShippingZoneId};

#[async_trait]
pub trait ShippingRateRepository: Send + Sync {
    async fn save(&self, rate: &ShippingRate) -> Result<(), ShippingError>;
    async fn find_by_id(&self, id: ShippingRateId) -> Result<Option<ShippingRate>, ShippingError>;
    async fn find_by_method_and_zone(
        &self,
        method_id: ShippingMethodId,
        zone_id: ShippingZoneId,
    ) -> Result<Vec<ShippingRate>, ShippingError>;
    async fn find_by_zone(
        &self,
        zone_id: ShippingZoneId,
    ) -> Result<Vec<ShippingRate>, ShippingError>;
    async fn update(&self, rate: &ShippingRate) -> Result<(), ShippingError>;
    async fn delete(&self, id: ShippingRateId) -> Result<(), ShippingError>;
}
