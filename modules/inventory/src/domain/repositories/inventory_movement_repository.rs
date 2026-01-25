// InventoryMovementRepository trait - repository for stock history/movement operations

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::domain::entities::InventoryMovement;
use crate::domain::value_objects::StockId;
use crate::InventoryError;

/// Query parameters for listing movements with filters
#[derive(Debug, Clone, Default)]
pub struct MovementQuery {
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Filter by specific stock ID
    pub stock_id: Option<StockId>,
    /// Filter by movement type (e.g., "in", "out", "adjustment")
    pub movement_type: Option<String>,
    /// Filter movements from this date (inclusive)
    pub from_date: Option<DateTime<Utc>>,
    /// Filter movements to this date (inclusive)
    pub to_date: Option<DateTime<Utc>>,
    /// Pagination: page number (1-indexed)
    pub page: i64,
    /// Pagination: items per page
    pub page_size: i64,
}

/// Repository trait for InventoryMovement (stock history) persistence operations.
/// Records all stock changes for audit and cost tracking purposes.
#[async_trait]
pub trait InventoryMovementRepository: Send + Sync {
    /// Saves a new movement to the repository
    async fn save(&self, movement: &InventoryMovement) -> Result<(), InventoryError>;

    /// Finds all movements for a specific stock record
    /// Results are ordered by created_at DESC (most recent first) for stock history reporting
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

    /// Counts movements for a specific stock record
    async fn count_by_stock_id(&self, stock_id: StockId) -> Result<i64, InventoryError>;

    /// Finds movements for a stock record within a date range
    async fn find_by_stock_id_and_date_range(
        &self,
        stock_id: StockId,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<InventoryMovement>, InventoryError>;

    /// Counts movements for a stock record within a date range
    async fn count_by_stock_id_and_date_range(
        &self,
        stock_id: StockId,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<i64, InventoryError>;

    /// Finds movements by reference type and ID (e.g., order, adjustment, transfer)
    async fn find_by_reference(
        &self,
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<Vec<InventoryMovement>, InventoryError>;

    /// Finds movements with filters for reporting purposes
    async fn find_with_filters(
        &self,
        query: &MovementQuery,
    ) -> Result<Vec<InventoryMovement>, InventoryError>;

    /// Counts movements matching the given filters
    async fn count_with_filters(&self, query: &MovementQuery) -> Result<i64, InventoryError>;

    /// Calculates the weighted average cost for a stock record based on movement history.
    /// Uses the formula: sum(quantity * unit_cost) / sum(quantity) for incoming movements.
    /// Returns None if no movements with cost information exist.
    async fn calculate_weighted_average_cost(
        &self,
        stock_id: StockId,
    ) -> Result<Option<Decimal>, InventoryError>;
}
