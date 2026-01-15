// AdjustmentRepository trait - repository for stock adjustment operations

use async_trait::async_trait;

use crate::domain::entities::StockAdjustment;
use crate::domain::value_objects::AdjustmentId;
use crate::InventoryError;
use identity::StoreId;

/// Repository trait for StockAdjustment persistence operations.
/// Handles adjustment documents with approval workflow.
#[async_trait]
pub trait AdjustmentRepository: Send + Sync {
    /// Saves a new adjustment to the repository
    async fn save(&self, adjustment: &StockAdjustment) -> Result<(), InventoryError>;

    /// Finds an adjustment by its unique ID (without items)
    async fn find_by_id(&self, id: AdjustmentId) -> Result<Option<StockAdjustment>, InventoryError>;

    /// Finds an adjustment by its unique ID with all items loaded
    async fn find_by_id_with_items(&self, id: AdjustmentId) -> Result<Option<StockAdjustment>, InventoryError>;

    /// Finds all adjustments for a specific store
    /// Results are ordered by created_at DESC
    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<StockAdjustment>, InventoryError>;

    /// Updates an existing adjustment
    async fn update(&self, adjustment: &StockAdjustment) -> Result<(), InventoryError>;

    /// Generates a unique adjustment number for a store
    /// Format: ADJ-{STORE_CODE}-{SEQUENCE}
    async fn generate_adjustment_number(&self, store_id: StoreId) -> Result<String, InventoryError>;
}
