// GetAdjustmentUseCase - retrieves an adjustment by ID with full details

use std::sync::Arc;

use crate::application::dtos::responses::{AdjustmentDetailResponse, AdjustmentItemResponse};
use crate::domain::repositories::AdjustmentRepository;
use crate::domain::value_objects::AdjustmentId;
use crate::InventoryError;

/// Use case for getting an adjustment by ID with full details
///
/// Retrieves the adjustment and all its items.
pub struct GetAdjustmentUseCase<A>
where
    A: AdjustmentRepository,
{
    adjustment_repo: Arc<A>,
}

impl<A> GetAdjustmentUseCase<A>
where
    A: AdjustmentRepository,
{
    /// Creates a new instance of GetAdjustmentUseCase
    pub fn new(adjustment_repo: Arc<A>) -> Self {
        Self { adjustment_repo }
    }

    /// Executes the use case to get an adjustment by ID
    ///
    /// # Arguments
    /// * `adjustment_id` - The UUID of the adjustment to retrieve
    ///
    /// # Returns
    /// AdjustmentDetailResponse with full adjustment details including items
    ///
    /// # Errors
    /// * `InventoryError::AdjustmentNotFound` - If the adjustment doesn't exist
    pub async fn execute(
        &self,
        adjustment_id: uuid::Uuid,
    ) -> Result<AdjustmentDetailResponse, InventoryError> {
        let adjustment_id_vo = AdjustmentId::from_uuid(adjustment_id);

        // Find the adjustment with items
        let adjustment = self
            .adjustment_repo
            .find_by_id_with_items(adjustment_id_vo)
            .await?
            .ok_or(InventoryError::AdjustmentNotFound(adjustment_id))?;

        // Build item responses
        let item_responses: Vec<AdjustmentItemResponse> = adjustment
            .items()
            .iter()
            .map(|item| AdjustmentItemResponse {
                id: item.id(),
                adjustment_id: item.adjustment_id().into_uuid(),
                stock_id: item.stock_id().into_uuid(),
                stock: None, // Could be loaded if needed
                quantity: item.quantity(),
                unit_cost: item.unit_cost(),
                balance_before: item.balance_before(),
                balance_after: item.balance_after(),
                notes: item.notes().map(|s| s.to_string()),
                created_at: item.created_at(),
            })
            .collect();

        Ok(AdjustmentDetailResponse {
            id: adjustment.id().into_uuid(),
            store_id: *adjustment.store_id().as_uuid(),
            adjustment_number: adjustment.adjustment_number().to_string(),
            adjustment_type: adjustment.adjustment_type().to_string(),
            adjustment_reason: adjustment.adjustment_reason().to_string(),
            status: adjustment.status().to_string(),
            created_by_id: adjustment.created_by_id().into_uuid(),
            approved_by_id: adjustment.approved_by_id().map(|id| id.into_uuid()),
            approved_at: adjustment.approved_at(),
            applied_at: adjustment.applied_at(),
            notes: adjustment.notes().map(|s| s.to_string()),
            attachments: Some(adjustment.attachments().clone()),
            items: item_responses,
            created_at: adjustment.created_at(),
            updated_at: adjustment.updated_at(),
        })
    }
}
