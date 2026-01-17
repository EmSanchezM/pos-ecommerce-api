// ListStockUseCase - lists stock records with pagination and filters

use std::sync::Arc;

use crate::application::dtos::responses::{PaginatedResponse, StockResponse};
use crate::domain::repositories::InventoryStockRepository;
use crate::domain::value_objects::ProductId;
use crate::InventoryError;
use identity::StoreId;

/// Query parameters for listing stock
#[derive(Debug, Clone)]
pub struct ListStockQuery {
    /// Filter by store ID
    pub store_id: Option<uuid::Uuid>,
    /// Filter by product ID
    pub product_id: Option<uuid::Uuid>,
    /// Filter to only show low stock items
    pub low_stock: bool,
    /// Page number (1-indexed)
    pub page: i64,
    /// Number of items per page
    pub page_size: i64,
}

impl Default for ListStockQuery {
    fn default() -> Self {
        Self {
            store_id: None,
            product_id: None,
            low_stock: false,
            page: 1,
            page_size: 20,
        }
    }
}

/// Use case for listing stock records with pagination and filters
pub struct ListStockUseCase<S>
where
    S: InventoryStockRepository,
{
    stock_repo: Arc<S>,
}

impl<S> ListStockUseCase<S>
where
    S: InventoryStockRepository,
{
    /// Creates a new instance of ListStockUseCase
    pub fn new(stock_repo: Arc<S>) -> Self {
        Self { stock_repo }
    }

    /// Executes the use case to list stock records
    ///
    /// # Arguments
    /// * `query` - Query parameters with filters and pagination
    ///
    /// # Returns
    /// Paginated response with stock records
    pub async fn execute(
        &self,
        query: ListStockQuery,
    ) -> Result<PaginatedResponse<StockResponse>, InventoryError> {
        // Validate and clamp pagination
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 100);

        // Convert UUIDs to value objects
        let store_id = query.store_id.map(StoreId::from_uuid);
        let product_id = query.product_id.map(ProductId::from_uuid);

        // Fetch stock records with pagination
        let (stocks, total_items) = self
            .stock_repo
            .find_paginated(store_id, product_id, query.low_stock, page, page_size)
            .await?;

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

        Ok(PaginatedResponse::new(
            stock_responses,
            page,
            page_size,
            total_items,
        ))
    }
}
