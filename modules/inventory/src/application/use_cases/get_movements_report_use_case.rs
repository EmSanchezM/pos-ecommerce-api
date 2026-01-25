// GetMovementsReportUseCase - generates paginated movements report with filters

use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::dtos::responses::{MovementResponse, PaginatedResponse};
use crate::domain::repositories::{InventoryMovementRepository, MovementQuery};
use crate::domain::value_objects::StockId;
use crate::InventoryError;

/// Query parameters for movements report
#[derive(Debug, Clone)]
pub struct MovementsReportQuery {
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Filter by stock ID
    pub stock_id: Option<Uuid>,
    /// Filter by movement type (e.g., "in", "out", "adjustment")
    pub movement_type: Option<String>,
    /// Filter movements from this date (inclusive)
    pub from_date: Option<DateTime<Utc>>,
    /// Filter movements to this date (inclusive)
    pub to_date: Option<DateTime<Utc>>,
    /// Page number (1-indexed)
    pub page: i64,
    /// Items per page
    pub page_size: i64,
}

impl Default for MovementsReportQuery {
    fn default() -> Self {
        Self {
            store_id: None,
            stock_id: None,
            movement_type: None,
            from_date: None,
            to_date: None,
            page: 1,
            page_size: 20,
        }
    }
}

/// Use case for generating paginated movements report
pub struct GetMovementsReportUseCase<M>
where
    M: InventoryMovementRepository,
{
    movement_repo: Arc<M>,
}

impl<M> GetMovementsReportUseCase<M>
where
    M: InventoryMovementRepository,
{
    pub fn new(movement_repo: Arc<M>) -> Self {
        Self { movement_repo }
    }

    /// Executes the use case to generate movements report
    ///
    /// # Arguments
    /// * `query` - Query parameters including filters and pagination
    ///
    /// # Returns
    /// PaginatedResponse with movement records
    pub async fn execute(
        &self,
        query: MovementsReportQuery,
    ) -> Result<PaginatedResponse<MovementResponse>, InventoryError> {
        // Convert to repository query
        let repo_query = MovementQuery {
            store_id: query.store_id,
            stock_id: query.stock_id.map(StockId::from_uuid),
            movement_type: query.movement_type,
            from_date: query.from_date,
            to_date: query.to_date,
            page: query.page,
            page_size: query.page_size,
        };

        // Get movements with filters
        let movements = self.movement_repo.find_with_filters(&repo_query).await?;

        // Count total matching movements
        let total_items = self.movement_repo.count_with_filters(&repo_query).await?;

        // Convert to response DTOs
        let items: Vec<MovementResponse> = movements
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

        Ok(PaginatedResponse::new(
            items,
            query.page,
            query.page_size,
            total_items,
        ))
    }
}
