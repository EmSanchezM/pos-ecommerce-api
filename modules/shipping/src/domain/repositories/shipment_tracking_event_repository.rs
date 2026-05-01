use async_trait::async_trait;

use crate::ShippingError;
use crate::domain::entities::ShipmentTrackingEvent;
use crate::domain::value_objects::ShipmentId;

#[async_trait]
pub trait ShipmentTrackingEventRepository: Send + Sync {
    async fn save(&self, event: &ShipmentTrackingEvent) -> Result<(), ShippingError>;
    async fn find_by_shipment(
        &self,
        shipment_id: ShipmentId,
    ) -> Result<Vec<ShipmentTrackingEvent>, ShippingError>;
}
