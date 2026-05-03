use async_trait::async_trait;
use uuid::Uuid;

use crate::BookingError;
use crate::domain::entities::Resource;
use crate::domain::value_objects::ResourceId;

#[async_trait]
pub trait ResourceRepository: Send + Sync {
    async fn save(&self, resource: &Resource) -> Result<(), BookingError>;
    async fn update(&self, resource: &Resource) -> Result<(), BookingError>;
    async fn find_by_id(&self, id: ResourceId) -> Result<Option<Resource>, BookingError>;
    async fn list_by_store(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<Resource>, BookingError>;
}
