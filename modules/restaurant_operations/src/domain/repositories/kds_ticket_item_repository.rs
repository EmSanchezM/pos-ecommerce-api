use async_trait::async_trait;

use crate::RestaurantOperationsError;
use crate::domain::entities::KdsTicketItem;
use crate::domain::value_objects::{KdsTicketId, KdsTicketItemId};

#[async_trait]
pub trait KdsTicketItemRepository: Send + Sync {
    async fn save(&self, item: &KdsTicketItem) -> Result<(), RestaurantOperationsError>;
    async fn update(&self, item: &KdsTicketItem) -> Result<(), RestaurantOperationsError>;
    async fn find_by_id(
        &self,
        id: KdsTicketItemId,
    ) -> Result<Option<KdsTicketItem>, RestaurantOperationsError>;
    async fn list_by_ticket(
        &self,
        ticket_id: KdsTicketId,
    ) -> Result<Vec<KdsTicketItem>, RestaurantOperationsError>;
}
