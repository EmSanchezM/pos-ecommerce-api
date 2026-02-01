// GetProductStockUseCase - retrieves stock for a product across all stores

use std::sync::Arc;

use crate::application::dtos::responses::{ListResponse, StockResponse};
use crate::domain::repositories::{InventoryStockRepository, ProductRepository};
use crate::domain::value_objects::ProductId;
use crate::InventoryError;

/// Use case for getting stock levels for a product across all stores
pub struct GetProductStockUseCase<S, P>
where
    S: InventoryStockRepository,
    P: ProductRepository,
{
    stock_repo: Arc<S>,
    product_repo: Arc<P>,
}

impl<S, P> GetProductStockUseCase<S, P>
where
    S: InventoryStockRepository,
    P: ProductRepository,
{
    /// Creates a new instance of GetProductStockUseCase
    pub fn new(stock_repo: Arc<S>, product_repo: Arc<P>) -> Self {
        Self {
            stock_repo,
            product_repo,
        }
    }

    /// Executes the use case to get stock for a product across all stores
    ///
    /// # Arguments
    /// * `product_id` - The UUID of the product
    ///
    /// # Returns
    /// ListResponse containing stock records for the product across all stores
    ///
    /// # Errors
    /// * `InventoryError::ProductNotFound` - If the product doesn't exist
    pub async fn execute(
        &self,
        product_id: uuid::Uuid,
    ) -> Result<ListResponse<StockResponse>, InventoryError> {
        let product_id_vo = ProductId::from_uuid(product_id);

        // Validate product exists
        if self.product_repo.find_by_id(product_id_vo).await?.is_none() {
            return Err(InventoryError::ProductNotFound(product_id));
        }

        // Find all stock for this product
        let stocks = self.stock_repo.find_by_product(product_id_vo).await?;

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
