// ListTransfersUseCase - lists stock transfers with filtering

use std::sync::Arc;

use crate::InventoryError;
use crate::application::dtos::responses::TransferResponse;
use crate::domain::entities::StockTransfer;
use crate::domain::repositories::TransferRepository;
use identity::StoreId;
use serde::Deserialize;

/// Query parameters for listing transfers
#[derive(Debug, Clone, Deserialize)]
pub struct ListTransfersQuery {
    /// Filter by store ID (shows transfers where store is source or destination)
    pub store_id: Option<uuid::Uuid>,
    /// Filter by direction relative to store_id: "outgoing", "incoming", or "all" (default)
    pub direction: Option<String>,
    /// Filter by status
    pub status: Option<String>,
}

/// Use case for listing stock transfers
pub struct ListTransfersUseCase<T>
where
    T: TransferRepository,
{
    transfer_repo: Arc<T>,
}

impl<T> ListTransfersUseCase<T>
where
    T: TransferRepository,
{
    pub fn new(transfer_repo: Arc<T>) -> Self {
        Self { transfer_repo }
    }

    pub async fn execute(
        &self,
        query: ListTransfersQuery,
    ) -> Result<Vec<TransferResponse>, InventoryError> {
        let transfers = if let Some(store_uuid) = query.store_id {
            let store_id = StoreId::from_uuid(store_uuid);
            let direction = query.direction.as_deref().unwrap_or("all");

            match direction {
                "outgoing" => self.transfer_repo.find_outgoing_by_store(store_id).await?,
                "incoming" => self.transfer_repo.find_incoming_by_store(store_id).await?,
                _ => self.transfer_repo.find_by_store(store_id).await?,
            }
        } else {
            // Without store_id filter, we can't list all - return empty
            // In practice, this should require a store_id
            return Ok(vec![]);
        };

        // Filter by status if provided
        let filtered: Vec<&StockTransfer> = if let Some(ref status) = query.status {
            transfers
                .iter()
                .filter(|t| t.status().to_string() == *status)
                .collect()
        } else {
            transfers.iter().collect()
        };

        Ok(filtered.iter().map(|t| Self::to_response(t)).collect())
    }

    fn to_response(transfer: &StockTransfer) -> TransferResponse {
        TransferResponse {
            id: transfer.id().into_uuid(),
            transfer_number: transfer.transfer_number().to_string(),
            from_store_id: transfer.from_store_id().into_uuid(),
            to_store_id: transfer.to_store_id().into_uuid(),
            status: transfer.status().to_string(),
            requested_date: transfer.requested_date(),
            shipped_date: transfer.shipped_date(),
            received_date: transfer.received_date(),
            requested_by_id: transfer.requested_by_id().into_uuid(),
            shipped_by_id: transfer.shipped_by_id().map(|id| id.into_uuid()),
            received_by_id: transfer.received_by_id().map(|id| id.into_uuid()),
            notes: transfer.notes().map(|s| s.to_string()),
            shipping_method: transfer.shipping_method().map(|s| s.to_string()),
            tracking_number: transfer.tracking_number().map(|s| s.to_string()),
            item_count: transfer.items().len() as i32,
            created_at: transfer.created_at(),
            updated_at: transfer.updated_at(),
        }
    }
}
