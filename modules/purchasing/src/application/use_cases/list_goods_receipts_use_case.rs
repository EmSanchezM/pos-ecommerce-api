// ListGoodsReceiptsUseCase - lists goods receipts with pagination and filters

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::responses::GoodsReceiptResponse;
use crate::domain::repositories::{GoodsReceiptFilter, GoodsReceiptRepository};
use crate::domain::value_objects::{GoodsReceiptStatus, PurchaseOrderId};
use crate::PurchasingError;
use inventory::PaginatedResponse;
use identity::StoreId;

/// Query parameters for listing goods receipts
#[derive(Debug, Clone, Default)]
pub struct ListGoodsReceiptsQuery {
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Filter by purchase order ID
    pub purchase_order_id: Option<Uuid>,
    /// Filter by status
    pub status: Option<String>,
    /// Page number (1-indexed)
    pub page: i64,
    /// Number of items per page
    pub page_size: i64,
}

/// Use case for listing goods receipts with pagination and filters
pub struct ListGoodsReceiptsUseCase<G>
where
    G: GoodsReceiptRepository,
{
    receipt_repo: Arc<G>,
}

impl<G> ListGoodsReceiptsUseCase<G>
where
    G: GoodsReceiptRepository,
{
    /// Creates a new instance of ListGoodsReceiptsUseCase
    pub fn new(receipt_repo: Arc<G>) -> Self {
        Self { receipt_repo }
    }

    /// Executes the use case to list goods receipts
    ///
    /// # Arguments
    /// * `query` - Query parameters with filters and pagination
    ///
    /// # Returns
    /// Paginated response with goods receipts
    pub async fn execute(
        &self,
        query: ListGoodsReceiptsQuery,
    ) -> Result<PaginatedResponse<GoodsReceiptResponse>, PurchasingError> {
        // Validate and clamp pagination
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 100);

        // Parse status if provided
        let status = query
            .status
            .map(|s| {
                s.parse::<GoodsReceiptStatus>()
                    .map_err(|_| PurchasingError::InvalidGoodsReceiptStatus)
            })
            .transpose()?;

        // Build filter
        let filter = GoodsReceiptFilter {
            store_id: query.store_id.map(StoreId::from_uuid),
            purchase_order_id: query.purchase_order_id.map(PurchaseOrderId::from_uuid),
            status,
        };

        // Fetch receipts with pagination
        let (receipts, total_items) = self
            .receipt_repo
            .find_paginated(filter, page, page_size)
            .await?;

        // Convert to response DTOs
        let receipt_responses: Vec<GoodsReceiptResponse> = receipts
            .into_iter()
            .map(|r| GoodsReceiptResponse {
                id: r.id().into_uuid(),
                receipt_number: r.receipt_number().to_string(),
                purchase_order_id: r.purchase_order_id().into_uuid(),
                store_id: r.store_id().into_uuid(),
                receipt_date: r.receipt_date(),
                status: r.status().to_string(),
                notes: r.notes().map(|s| s.to_string()),
                received_by_id: r.received_by_id().into_uuid(),
                confirmed_by_id: r.confirmed_by_id().map(|id| id.into_uuid()),
                confirmed_at: r.confirmed_at(),
                created_at: r.created_at(),
                updated_at: r.updated_at(),
            })
            .collect();

        Ok(PaginatedResponse::new(
            receipt_responses,
            page,
            page_size,
            total_items,
        ))
    }
}
