// ReservationRepository trait - repository for inventory reservation operations

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::InventoryReservation;
use crate::domain::value_objects::{ReservationId, StockId};
use crate::InventoryError;

/// Repository trait for InventoryReservation persistence operations.
/// Handles temporary stock holds for shopping carts, orders, and quotes.
#[async_trait]
pub trait ReservationRepository: Send + Sync {
    /// Saves a new reservation to the repository
    async fn save(&self, reservation: &InventoryReservation) -> Result<(), InventoryError>;

    /// Finds a reservation by its unique ID
    async fn find_by_id(&self, id: ReservationId) -> Result<Option<InventoryReservation>, InventoryError>;

    /// Finds all reservations for a specific stock record
    async fn find_by_stock_id(&self, stock_id: StockId) -> Result<Vec<InventoryReservation>, InventoryError>;

    /// Finds reservations by reference type and ID (e.g., cart, order, quote)
    async fn find_by_reference(
        &self,
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<Vec<InventoryReservation>, InventoryError>;

    /// Finds all expired pending reservations
    /// Returns reservations where status is Pending and expires_at < now
    async fn find_expired(&self) -> Result<Vec<InventoryReservation>, InventoryError>;

    /// Updates an existing reservation
    async fn update(&self, reservation: &InventoryReservation) -> Result<(), InventoryError>;

    /// Deletes a reservation by ID
    async fn delete(&self, id: ReservationId) -> Result<(), InventoryError>;
}
