// GetStockUseCase - retrieves a stock record by ID with detailed information

use std::sync::Arc;

use crate::application::dtos::responses::{
    StockDetailResponse, VariantResponse, ProductResponse
};
use crate::domain::repositories::{InventoryStockRepository, ProductRepository};
use crate::domain::value_objects::StockId;
use crate::InventoryError;

/// Use case for getting a stock record by ID with full details
pub struct GetStockUseCase<S, P>
where
    S: InventoryStockRepository,
    P: ProductRepository,
{
    stock_repo: Arc<S>,
    product_repo: Arc<P>,
}

impl<S, P> GetStockUseCase<S, P>
where
    S: InventoryStockRepository,
    P: ProductRepository,
{
    /// Creates a new instance of GetStockUseCase
    pub fn new(stock_repo: Arc<S>, product_repo: Arc<P>) -> Self {
        Self {
            stock_repo,
            product_repo,
        }
    }

    /// Executes the use case to get a stock record by ID
    ///
    /// # Arguments
    /// * `stock_id` - The UUID of the stock record to retrieve
    ///
    /// # Returns
    /// StockDetailResponse with full stock details including product/variant info
    ///
    /// # Errors
    /// * `InventoryError::StockNotFound` - If the stock record doesn't exist
    pub async fn execute(
        &self,
        stock_id: uuid::Uuid,
    ) -> Result<StockDetailResponse, InventoryError> {
        let stock_id_vo = StockId::from_uuid(stock_id);

        // Find the stock record
        let stock = self
            .stock_repo
            .find_by_id(stock_id_vo)
            .await?
            .ok_or(InventoryError::StockNotFound(stock_id))?;

        // Get product info if available
        let product_response = if let Some(product_id) = stock.product_id() {
            self.product_repo
                .find_by_id(product_id)
                .await?
                .map(|p| ProductResponse {
                    id: p.id().into_uuid(),
                    sku: p.sku().to_string(),
                    barcode: p.barcode().map(|b| b.to_string()),
                    name: p.name().to_string(),
                    description: p.description().map(|s| s.to_string()),
                    category_id: p.category_id().map(|id| id.into_uuid()),
                    brand: p.brand().map(|s| s.to_string()),
                    unit_of_measure: p.unit_of_measure().to_string(),
                    base_price: p.base_price(),
                    cost_price: p.cost_price(),
                    currency: p.currency().to_string(),
                    is_perishable: p.is_perishable(),
                    is_trackable: p.is_trackable(),
                    has_variants: p.has_variants(),
                    tax_rate: p.tax_rate(),
                    tax_included: p.tax_included(),
                    is_active: p.is_active(),
                    created_at: p.created_at(),
                    updated_at: p.updated_at(),
                })
        } else {
            None
        };

        // Get variant info if available
        let variant_response = if let Some(variant_id) = stock.variant_id() {
            self.product_repo
                .find_variant_by_id(variant_id)
                .await?
                .map(|v| VariantResponse {
                    id: v.id().into_uuid(),
                    product_id: v.product_id().into_uuid(),
                    sku: v.sku().to_string(),
                    barcode: v.barcode().map(|b| b.to_string()),
                    name: v.name().to_string(),
                    variant_attributes: v.variant_attributes().clone(),
                    price: v.price(),
                    cost_price: v.cost_price(),
                    effective_price: v.price().unwrap_or_default(),
                    effective_cost: v.cost_price().unwrap_or_default(),
                    is_active: v.is_active(),
                    created_at: v.created_at(),
                    updated_at: v.updated_at(),
                })
        } else {
            None
        };

        let available = stock.available_quantity();
        let is_low = available <= stock.min_stock_level();

        Ok(StockDetailResponse {
            id: stock.id().into_uuid(),
            store_id: *stock.store_id().as_uuid(),
            product_id: stock.product_id().map(|id| id.into_uuid()),
            variant_id: stock.variant_id().map(|id| id.into_uuid()),
            product: product_response,
            variant: variant_response,
            quantity: stock.quantity(),
            reserved_quantity: stock.reserved_quantity(),
            available_quantity: available,
            version: stock.version(),
            min_stock_level: stock.min_stock_level(),
            max_stock_level: stock.max_stock_level(),
            is_low_stock: is_low,
            weighted_average_cost: None, // Could be calculated from movements
            created_at: stock.created_at(),
            updated_at: stock.updated_at(),
        })
    }
}
