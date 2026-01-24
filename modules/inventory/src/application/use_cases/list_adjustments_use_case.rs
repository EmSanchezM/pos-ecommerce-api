// ListAdjustmentsUseCase - lists adjustments with pagination and filters

use std::sync::Arc;

use identity::StoreId;

use crate::application::dtos::responses::{AdjustmentResponse, PaginatedResponse};
use crate::domain::repositories::AdjustmentRepository;
use crate::InventoryError;

/// Query parameters for listing adjustments
#[derive(Debug, Clone)]
pub struct ListAdjustmentsQuery {
    /// Filter by store ID
    pub store_id: Option<uuid::Uuid>,
    /// Filter by status (draft, pending_approval, approved, rejected, applied)
    pub status: Option<String>,
    /// Page number (1-indexed)
    pub page: i64,
    /// Number of items per page
    pub page_size: i64,
}

impl Default for ListAdjustmentsQuery {
    fn default() -> Self {
        Self {
            store_id: None,
            status: None,
            page: 1,
            page_size: 20,
        }
    }
}

/// Use case for listing adjustments with pagination and filters
pub struct ListAdjustmentsUseCase<A>
where
    A: AdjustmentRepository,
{
    adjustment_repo: Arc<A>,
}

impl<A> ListAdjustmentsUseCase<A>
where
    A: AdjustmentRepository,
{
    /// Creates a new instance of ListAdjustmentsUseCase
    pub fn new(adjustment_repo: Arc<A>) -> Self {
        Self { adjustment_repo }
    }

    /// Executes the use case to list adjustments
    ///
    /// # Arguments
    /// * `query` - Query parameters with filters and pagination
    ///
    /// # Returns
    /// Paginated response with adjustments
    pub async fn execute(
        &self,
        query: ListAdjustmentsQuery,
    ) -> Result<PaginatedResponse<AdjustmentResponse>, InventoryError> {
        // Validate and clamp pagination
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 100);

        // Convert store_id to StoreId value object
        let store_id_vo = query.store_id.map(StoreId::from_uuid);

        // Fetch adjustments with pagination
        let (adjustments, total_items) = self
            .adjustment_repo
            .find_paginated(
                store_id_vo,
                query.status.as_deref(),
                page,
                page_size,
            )
            .await?;

        // Convert to response DTOs
        let adjustment_responses: Vec<AdjustmentResponse> = adjustments
            .into_iter()
            .map(|a| AdjustmentResponse {
                id: a.id().into_uuid(),
                store_id: *a.store_id().as_uuid(),
                adjustment_number: a.adjustment_number().to_string(),
                adjustment_type: a.adjustment_type().to_string(),
                adjustment_reason: a.adjustment_reason().to_string(),
                status: a.status().to_string(),
                created_by_id: a.created_by_id().into_uuid(),
                approved_by_id: a.approved_by_id().map(|id| id.into_uuid()),
                approved_at: a.approved_at(),
                applied_at: a.applied_at(),
                notes: a.notes().map(|s| s.to_string()),
                item_count: a.items().len() as i32,
                created_at: a.created_at(),
                updated_at: a.updated_at(),
            })
            .collect();

        Ok(PaginatedResponse::new(
            adjustment_responses,
            page,
            page_size,
            total_items,
        ))
    }
}
