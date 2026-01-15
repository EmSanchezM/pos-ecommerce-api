// InventoryMovementRepository trait - repository for Kardex/movement operations

use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::domain::entities::InventoryMovement;
use crate::domain::value_objects::StockId;
use crate::InventoryError;

/// Repository trait for InventoryMovement (Kardex) persistence operations.
/// Records all stock changes for audit and cost tracking purposes.
#[async_trait]
pub trait InventoryMovementRepository: Send + Sync {
    /// Saves a new movement to the repository
    async fn save(&self, movement: &InventoryMovement) -> Result<(), InventoryError>;

    /// Finds all movements for a specific stock record
    /// Results are ordered by created_at DESC (most recent first) for Kardex reporting
    /// 
    /// # Arguments
    /// * `stock_id` - The stock record ID
    /// * `limit` - Maximum number of records to return
    /// * `offset` - Number of records to skip (for pagination)
    async fn find_by_stock_id(
        &self,
        stock_id: StockId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<InventoryMovement>, InventoryError>;

    /// Finds movements by reference type and ID (e.g., order, adjustment, transfer)
    async fn find_by_reference(
        &self,
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<Vec<InventoryMovement>, InventoryError>;

    /// Calculates the weighted average cost for a stock record based on movement history.
    /// Uses the formula: sum(quantity * unit_cost) / sum(quantity) for incoming movements.
    /// Returns None if no movements with cost information exist.
    async fn calculate_weighted_average_cost(
        &self,
        stock_id: StockId,
    ) -> Result<Option<Decimal>, InventoryError>;
}
