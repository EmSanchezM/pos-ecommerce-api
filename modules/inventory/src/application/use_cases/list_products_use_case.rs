// ListProductsUseCase - lists products with pagination and filters

use std::sync::Arc;

use crate::application::dtos::responses::{PaginatedResponse, ProductResponse};
use crate::domain::repositories::ProductRepository;
use crate::domain::value_objects::CategoryId;
use crate::InventoryError;

/// Query parameters for listing products
#[derive(Debug, Clone)]
pub struct ListProductsQuery {
    /// Filter by category ID
    pub category_id: Option<uuid::Uuid>,
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Search term for name/description
    pub search: Option<String>,
    /// Page number (1-indexed)
    pub page: i64,
    /// Number of items per page
    pub page_size: i64,
}

impl Default for ListProductsQuery {
    fn default() -> Self {
        Self {
            category_id: None,
            is_active: None,
            search: None,
            page: 1,
            page_size: 20,
        }
    }
}

/// Use case for listing products with pagination and filters
pub struct ListProductsUseCase<P>
where
    P: ProductRepository,
{
    product_repo: Arc<P>,
}

impl<P> ListProductsUseCase<P>
where
    P: ProductRepository,
{
    /// Creates a new instance of ListProductsUseCase
    pub fn new(product_repo: Arc<P>) -> Self {
        Self { product_repo }
    }

    /// Executes the use case to list products
    ///
    /// # Arguments
    /// * `query` - Query parameters with filters and pagination
    ///
    /// # Returns
    /// Paginated response with products
    pub async fn execute(
        &self,
        query: ListProductsQuery,
    ) -> Result<PaginatedResponse<ProductResponse>, InventoryError> {
        // Validate and clamp pagination
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 100);

        // Convert category_id to domain type
        let category_id = query.category_id.map(CategoryId::from_uuid);

        // Fetch products with pagination
        let (products, total_items) = self
            .product_repo
            .find_paginated(
                category_id,
                query.is_active,
                query.search.as_deref(),
                page,
                page_size,
            )
            .await?;

        // Convert to response DTOs
        let product_responses: Vec<ProductResponse> = products
            .into_iter()
            .map(|p| ProductResponse {
                id: p.id().into_uuid(),
                sku: p.sku().as_str().to_string(),
                barcode: p.barcode().map(|b| b.as_str().to_string()),
                name: p.name().to_string(),
                description: p.description().map(|s| s.to_string()),
                category_id: p.category_id().map(|id| id.into_uuid()),
                brand: p.brand().map(|s| s.to_string()),
                unit_of_measure: p.unit_of_measure().to_string(),
                base_price: p.base_price(),
                cost_price: p.cost_price(),
                currency: p.currency().as_str().to_string(),
                is_perishable: p.is_perishable(),
                is_trackable: p.is_trackable(),
                has_variants: p.has_variants(),
                tax_rate: p.tax_rate(),
                tax_included: p.tax_included(),
                is_active: p.is_active(),
                created_at: p.created_at(),
                updated_at: p.updated_at(),
            })
            .collect();

        Ok(PaginatedResponse::new(
            product_responses,
            page,
            page_size,
            total_items,
        ))
    }
}
