use async_trait::async_trait;
use uuid::Uuid;

use crate::BookingError;
use crate::domain::entities::Service;
use crate::domain::value_objects::{ResourceId, ServiceId};

#[async_trait]
pub trait ServiceRepository: Send + Sync {
    async fn save(&self, service: &Service) -> Result<(), BookingError>;
    async fn update(&self, service: &Service) -> Result<(), BookingError>;
    async fn find_by_id(&self, id: ServiceId) -> Result<Option<Service>, BookingError>;
    async fn list_by_store(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<Service>, BookingError>;

    /// Replace the full set of resources eligible to perform `service_id`.
    async fn assign_resources(
        &self,
        service_id: ServiceId,
        resource_ids: &[ResourceId],
    ) -> Result<(), BookingError>;

    async fn find_eligible_resources(
        &self,
        service_id: ServiceId,
    ) -> Result<Vec<ResourceId>, BookingError>;
}
