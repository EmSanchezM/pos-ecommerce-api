// ListVendorsUseCase - lists vendors with pagination and filters

use std::sync::Arc;

use crate::application::dtos::responses::VendorResponse;
use crate::domain::repositories::{VendorFilter, VendorRepository};
use crate::PurchasingError;
use inventory::PaginatedResponse;

/// Query parameters for listing vendors
#[derive(Debug, Clone, Default)]
pub struct ListVendorsQuery {
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Search by name or code
    pub search: Option<String>,
    /// Page number (1-indexed)
    pub page: i64,
    /// Number of items per page
    pub page_size: i64,
}

/// Use case for listing vendors with pagination and filters
pub struct ListVendorsUseCase<V>
where
    V: VendorRepository,
{
    vendor_repo: Arc<V>,
}

impl<V> ListVendorsUseCase<V>
where
    V: VendorRepository,
{
    /// Creates a new instance of ListVendorsUseCase
    pub fn new(vendor_repo: Arc<V>) -> Self {
        Self { vendor_repo }
    }

    /// Executes the use case to list vendors
    ///
    /// # Arguments
    /// * `query` - Query parameters with filters and pagination
    ///
    /// # Returns
    /// Paginated response with vendors
    pub async fn execute(
        &self,
        query: ListVendorsQuery,
    ) -> Result<PaginatedResponse<VendorResponse>, PurchasingError> {
        // Validate and clamp pagination
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 100);

        // Build filter
        let filter = VendorFilter {
            is_active: query.is_active,
            search: query.search,
        };

        // Fetch vendors with pagination
        let (vendors, total_items) = self
            .vendor_repo
            .find_paginated(filter, page, page_size)
            .await?;

        // Convert to response DTOs
        let vendor_responses: Vec<VendorResponse> = vendors
            .into_iter()
            .map(|v| VendorResponse {
                id: v.id().into_uuid(),
                code: v.code().to_string(),
                name: v.name().to_string(),
                legal_name: v.legal_name().to_string(),
                tax_id: v.tax_id().to_string(),
                email: v.email().map(|s| s.to_string()),
                phone: v.phone().map(|s| s.to_string()),
                address: v.address().map(|s| s.to_string()),
                payment_terms_days: v.payment_terms_days(),
                currency: v.currency().as_str().to_string(),
                is_active: v.is_active(),
                notes: v.notes().map(|s| s.to_string()),
                created_at: v.created_at(),
                updated_at: v.updated_at(),
            })
            .collect();

        Ok(PaginatedResponse::new(
            vendor_responses,
            page,
            page_size,
            total_items,
        ))
    }
}
