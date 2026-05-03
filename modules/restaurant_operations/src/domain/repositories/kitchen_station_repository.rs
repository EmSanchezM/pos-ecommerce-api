use async_trait::async_trait;
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::entities::KitchenStation;
use crate::domain::value_objects::KitchenStationId;

#[async_trait]
pub trait KitchenStationRepository: Send + Sync {
    async fn save(&self, station: &KitchenStation) -> Result<(), RestaurantOperationsError>;
    async fn update(&self, station: &KitchenStation) -> Result<(), RestaurantOperationsError>;
    async fn find_by_id(
        &self,
        id: KitchenStationId,
    ) -> Result<Option<KitchenStation>, RestaurantOperationsError>;
    async fn list_by_store(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<KitchenStation>, RestaurantOperationsError>;
}
