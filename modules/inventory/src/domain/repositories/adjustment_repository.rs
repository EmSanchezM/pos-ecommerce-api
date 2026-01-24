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

    /// Finds adjustments with pagination and optional filters
    ///
    /// # Arguments
    /// * `store_id` - Optional filter by store ID
    /// * `status` - Optional filter by status (draft, pending_approval, approved, rejected, applied)
    /// * `page` - Page number (1-indexed)
    /// * `page_size` - Number of items per page
    ///
    /// # Returns
    /// Tuple of (adjustments, total_count) for pagination
    async fn find_paginated(
        &self,
        store_id: Option<StoreId>,
        status: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<StockAdjustment>, i64), InventoryError>;
}
