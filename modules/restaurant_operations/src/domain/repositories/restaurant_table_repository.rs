use async_trait::async_trait;
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::entities::RestaurantTable;
use crate::domain::value_objects::RestaurantTableId;

#[async_trait]
pub trait RestaurantTableRepository: Send + Sync {
    async fn save(&self, table: &RestaurantTable) -> Result<(), RestaurantOperationsError>;
    async fn update(&self, table: &RestaurantTable) -> Result<(), RestaurantOperationsError>;
    async fn find_by_id(
        &self,
        id: RestaurantTableId,
    ) -> Result<Option<RestaurantTable>, RestaurantOperationsError>;
    async fn list_by_store(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<RestaurantTable>, RestaurantOperationsError>;
}
