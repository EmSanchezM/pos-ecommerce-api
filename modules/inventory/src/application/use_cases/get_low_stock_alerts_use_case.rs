// GetLowStockAlertsUseCase - retrieves products with low stock for alerts

use std::sync::Arc;

use crate::application::dtos::responses::StockResponse;
use crate::domain::repositories::InventoryStockRepository;
use crate::InventoryError;
use identity::StoreId;

/// Use case for retrieving low stock alerts for a store.
///
/// Returns all stock records where available_quantity <= min_stock_level,
/// useful for generating reorder alerts and notifications.
pub struct GetLowStockAlertsUseCase<S>
where
    S: InventoryStockRepository,
{
    stock_repo: Arc<S>,
}

impl<S> GetLowStockAlertsUseCase<S>
where
    S: InventoryStockRepository,
{
    /// Creates a new instance of GetLowStockAlertsUseCase
    pub fn new(stock_repo: Arc<S>) -> Self {
        Self { stock_repo }
    }

    /// Executes the use case to get low stock alerts
    ///
    /// # Arguments
    /// * `store_id` - The store ID to check for low stock
    ///
    /// # Returns
    /// Vector of StockResponse for items with low stock
    pub async fn execute(
        &self,
        store_id: uuid::Uuid,
    ) -> Result<Vec<StockResponse>, InventoryError> {
        let store_id = StoreId::from_uuid(store_id);

        // Fetch low stock items
        let stocks = self.stock_repo.find_low_stock(store_id).await?;

        // Convert to response DTOs
        let responses = stocks
            .into_iter()
            .map(|s| StockResponse {
                id: s.id().into_uuid(),
                store_id: s.store_id().into_uuid(),
                product_id: s.product_id().map(|id| id.into_uuid()),
                variant_id: s.variant_id().map(|id| id.into_uuid()),
                quantity: s.quantity(),
                reserved_quantity: s.reserved_quantity(),
                available_quantity: s.available_quantity(),
                version: s.version(),
                min_stock_level: s.min_stock_level(),
                max_stock_level: s.max_stock_level(),
                is_low_stock: true, // All items from find_low_stock are low stock
                created_at: s.created_at(),
                updated_at: s.updated_at(),
            })
            .collect();

        Ok(responses)
    }
}
