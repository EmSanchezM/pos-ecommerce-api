use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::entities::KdsTicket;
use crate::domain::value_objects::{
    KdsTicketId, KdsTicketStatus, KitchenStationId, RestaurantTableId,
};

#[derive(Debug, Clone, Default)]
pub struct ListKdsTicketsFilters {
    pub store_id: Option<Uuid>,
    pub station_id: Option<KitchenStationId>,
    pub table_id: Option<RestaurantTableId>,
    pub status: Option<KdsTicketStatus>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[async_trait]
pub trait KdsTicketRepository: Send + Sync {
    async fn save(&self, ticket: &KdsTicket) -> Result<(), RestaurantOperationsError>;
    async fn update(&self, ticket: &KdsTicket) -> Result<(), RestaurantOperationsError>;
    async fn find_by_id(
        &self,
        id: KdsTicketId,
    ) -> Result<Option<KdsTicket>, RestaurantOperationsError>;
    async fn list(
        &self,
        filters: ListKdsTicketsFilters,
    ) -> Result<Vec<KdsTicket>, RestaurantOperationsError>;
    /// Returns `MAX(ticket_number) + 1` for the store, or `1` if none. The
    /// implementation must run inside a transaction with `FOR UPDATE` on the
    /// existing rows to avoid two concurrent creates picking the same number.
    async fn next_ticket_number(&self, store_id: Uuid) -> Result<i32, RestaurantOperationsError>;
}
