// GetStoreInventoryUseCase - retrieves all stock records for a specific store

use std::sync::Arc;

use crate::application::dtos::responses::{ListResponse, StockResponse};
use crate::domain::repositories::InventoryStockRepository;
use crate::InventoryError;
use identity::StoreId;

/// Use case for getting all inventory for a specific store
pub struct GetStoreInventoryUseCase<S>
where
    S: InventoryStockRepository,
{
    stock_repo: Arc<S>,
}

impl<S> GetStoreInventoryUseCase<S>
where
    S: InventoryStockRepository,
{
    /// Creates a new instance of GetStoreInventoryUseCase
    pub fn new(stock_repo: Arc<S>) -> Self {
        Self { stock_repo }
    }

    /// Executes the use case to get all inventory for a store
    ///
    /// # Arguments
    /// * `store_id` - The UUID of the store
    ///
    /// # Returns
    /// ListResponse containing stock records for the store
    pub async fn execute(
        &self,
        store_id: uuid::Uuid,
    ) -> Result<ListResponse<StockResponse>, InventoryError> {
        let store_id_vo = StoreId::from_uuid(store_id);

        // Find all stock for this store
        let stocks = self.stock_repo.find_by_store(store_id_vo).await?;

        // Convert to response DTOs
        let stock_responses: Vec<StockResponse> = stocks
            .into_iter()
            .map(|s| {
                let available = s.available_quantity();
                let is_low = available <= s.min_stock_level();
                StockResponse {
                    id: s.id().into_uuid(),
                    store_id: *s.store_id().as_uuid(),
                    product_id: s.product_id().map(|id| id.into_uuid()),
                    variant_id: s.variant_id().map(|id| id.into_uuid()),
                    quantity: s.quantity(),
                    reserved_quantity: s.reserved_quantity(),
                    available_quantity: available,
                    version: s.version(),
                    min_stock_level: s.min_stock_level(),
                    max_stock_level: s.max_stock_level(),
                    is_low_stock: is_low,
                    created_at: s.created_at(),
                    updated_at: s.updated_at(),
                }
            })
            .collect();

        Ok(ListResponse::new(stock_responses))
    }
}
