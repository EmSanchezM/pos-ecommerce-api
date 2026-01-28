// ListPurchaseOrdersUseCase - lists purchase orders with pagination and filters

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::responses::PurchaseOrderResponse;
use crate::domain::repositories::{PurchaseOrderFilter, PurchaseOrderRepository};
use crate::domain::value_objects::{PurchaseOrderStatus, VendorId};
use crate::PurchasingError;
use inventory::PaginatedResponse;
use identity::StoreId;

/// Query parameters for listing purchase orders
#[derive(Debug, Clone, Default)]
pub struct ListPurchaseOrdersQuery {
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Filter by vendor ID
    pub vendor_id: Option<Uuid>,
    /// Filter by status
    pub status: Option<String>,
    /// Search by order number
    pub search: Option<String>,
    /// Page number (1-indexed)
    pub page: i64,
    /// Number of items per page
    pub page_size: i64,
}

/// Use case for listing purchase orders with pagination and filters
pub struct ListPurchaseOrdersUseCase<P>
where
    P: PurchaseOrderRepository,
{
    order_repo: Arc<P>,
}

impl<P> ListPurchaseOrdersUseCase<P>
where
    P: PurchaseOrderRepository,
{
    /// Creates a new instance of ListPurchaseOrdersUseCase
    pub fn new(order_repo: Arc<P>) -> Self {
        Self { order_repo }
    }

    /// Executes the use case to list purchase orders
    ///
    /// # Arguments
    /// * `query` - Query parameters with filters and pagination
    ///
    /// # Returns
    /// Paginated response with purchase orders
    pub async fn execute(
        &self,
        query: ListPurchaseOrdersQuery,
    ) -> Result<PaginatedResponse<PurchaseOrderResponse>, PurchasingError> {
        // Validate and clamp pagination
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 100);

        // Parse status if provided
        let status = query
            .status
            .map(|s| {
                s.parse::<PurchaseOrderStatus>()
                    .map_err(|_| PurchasingError::InvalidPurchaseOrderStatus)
            })
            .transpose()?;

        // Build filter
        let filter = PurchaseOrderFilter {
            store_id: query.store_id.map(StoreId::from_uuid),
            vendor_id: query.vendor_id.map(VendorId::from_uuid),
            status,
            search: query.search,
        };

        // Fetch orders with pagination
        let (orders, total_items) = self
            .order_repo
            .find_paginated(filter, page, page_size)
            .await?;

        // Convert to response DTOs
        let order_responses: Vec<PurchaseOrderResponse> = orders
            .into_iter()
            .map(|o| PurchaseOrderResponse {
                id: o.id().into_uuid(),
                order_number: o.order_number().to_string(),
                store_id: o.store_id().into_uuid(),
                vendor_id: o.vendor_id().into_uuid(),
                status: o.status().to_string(),
                order_date: o.order_date(),
                expected_delivery_date: o.expected_delivery_date(),
                subtotal: o.subtotal(),
                tax_amount: o.tax_amount(),
                discount_amount: o.discount_amount(),
                total: o.total(),
                currency: o.currency().as_str().to_string(),
                payment_terms_days: o.payment_terms_days(),
                notes: o.notes().map(|s| s.to_string()),
                created_by_id: o.created_by_id().into_uuid(),
                created_at: o.created_at(),
                updated_at: o.updated_at(),
            })
            .collect();

        Ok(PaginatedResponse::new(
            order_responses,
            page,
            page_size,
            total_items,
        ))
    }
}
