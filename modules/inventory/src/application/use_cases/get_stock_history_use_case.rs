// GetStockHistoryUseCase - retrieves movement history for a stock record

use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::dtos::responses::{
    MovementResponse, ProductResponse, StockHistoryResponse, VariantResponse,
};
use crate::domain::repositories::{InventoryMovementRepository, InventoryStockRepository, ProductRepository};
use crate::domain::value_objects::StockId;
use crate::InventoryError;

/// Query parameters for stock history
#[derive(Debug, Clone)]
pub struct StockHistoryQuery {
    /// Stock record ID
    pub stock_id: Uuid,
    /// Filter movements from this date (inclusive)
    pub from_date: Option<DateTime<Utc>>,
    /// Filter movements to this date (inclusive)
    pub to_date: Option<DateTime<Utc>>,
    /// Page number (1-indexed)
    pub page: i64,
    /// Items per page
    pub page_size: i64,
}

/// Use case for getting stock history (movement ledger) for a specific stock record
pub struct GetStockHistoryUseCase<S, M, P>
where
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
    P: ProductRepository,
{
    stock_repo: Arc<S>,
    movement_repo: Arc<M>,
    product_repo: Arc<P>,
}

impl<S, M, P> GetStockHistoryUseCase<S, M, P>
where
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
    P: ProductRepository,
{
    pub fn new(stock_repo: Arc<S>, movement_repo: Arc<M>, product_repo: Arc<P>) -> Self {
        Self {
            stock_repo,
            movement_repo,
            product_repo,
        }
    }

    /// Executes the use case to get stock history
    ///
    /// # Arguments
    /// * `query` - Query parameters including stock_id, date filters, and pagination
    ///
    /// # Returns
    /// StockHistoryResponse with stock info and paginated movements
    pub async fn execute(&self, query: StockHistoryQuery) -> Result<StockHistoryResponse, InventoryError> {
        let stock_id = StockId::from_uuid(query.stock_id);
        let offset = (query.page - 1) * query.page_size;

        // Find the stock record
        let stock = self
            .stock_repo
            .find_by_id(stock_id)
            .await?
            .ok_or(InventoryError::StockNotFound(query.stock_id))?;

        // Get movements with optional date filtering
        let movements = self
            .movement_repo
            .find_by_stock_id_and_date_range(
                stock_id,
                query.from_date,
                query.to_date,
                query.page_size,
                offset,
            )
            .await?;

        // Count total movements
        let total_movements = self
            .movement_repo
            .count_by_stock_id_and_date_range(stock_id, query.from_date, query.to_date)
            .await?;

        // Calculate weighted average cost
        let weighted_avg_cost = self
            .movement_repo
            .calculate_weighted_average_cost(stock_id)
            .await?;

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

        // Convert movements to response DTOs
        let movement_responses: Vec<MovementResponse> = movements
            .into_iter()
            .map(|m| MovementResponse {
                id: m.id().into_uuid(),
                stock_id: m.stock_id().into_uuid(),
                movement_type: m.movement_type().to_string(),
                movement_reason: m.movement_reason().map(|s| s.to_string()),
                quantity: m.quantity(),
                unit_cost: m.unit_cost(),
                total_cost: m.total_cost(),
                currency: m.currency().to_string(),
                balance_after: m.balance_after(),
                reference_type: m.reference_type().map(|s| s.to_string()),
                reference_id: m.reference_id(),
                actor_id: m.actor_id().into_uuid(),
                notes: m.notes().map(|s| s.to_string()),
                metadata: Some(m.metadata().clone()),
                created_at: m.created_at(),
            })
            .collect();

        Ok(StockHistoryResponse {
            stock_id: stock.id().into_uuid(),
            product_id: stock.product_id().map(|id| id.into_uuid()),
            variant_id: stock.variant_id().map(|id| id.into_uuid()),
            product: product_response,
            variant: variant_response,
            current_quantity: stock.quantity(),
            current_reserved: stock.reserved_quantity(),
            current_available: stock.available_quantity(),
            weighted_average_cost: weighted_avg_cost,
            movements: movement_responses,
            total_movements,
        })
    }
}
