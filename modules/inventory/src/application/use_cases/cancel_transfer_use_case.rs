// CancelTransferUseCase - cancels a draft or pending transfer

use std::sync::Arc;

use crate::InventoryError;
use crate::application::dtos::responses::{TransferDetailResponse, TransferItemResponse};
use crate::domain::entities::StockTransfer;
use crate::domain::repositories::TransferRepository;
use crate::domain::value_objects::TransferId;

/// Use case for cancelling a stock transfer
pub struct CancelTransferUseCase<T>
where
    T: TransferRepository,
{
    transfer_repo: Arc<T>,
}

impl<T> CancelTransferUseCase<T>
where
    T: TransferRepository,
{
    pub fn new(transfer_repo: Arc<T>) -> Self {
        Self { transfer_repo }
    }

    pub async fn execute(&self, id: uuid::Uuid) -> Result<TransferDetailResponse, InventoryError> {
        let transfer_id = TransferId::from_uuid(id);
        let mut transfer = self
            .transfer_repo
            .find_by_id_with_items(transfer_id)
            .await?
            .ok_or(InventoryError::TransferNotFound(id))?;

        transfer.cancel()?;
        self.transfer_repo.update(&transfer).await?;

        Ok(Self::to_response(&transfer))
    }

    fn to_response(transfer: &StockTransfer) -> TransferDetailResponse {
        let items: Vec<TransferItemResponse> = transfer
            .items()
            .iter()
            .map(|item| TransferItemResponse {
                id: item.id(),
                transfer_id: item.transfer_id().into_uuid(),
                product_id: item.product_id().map(|id| id.into_uuid()),
                variant_id: item.variant_id().map(|id| id.into_uuid()),
                product: None,
                variant: None,
                quantity_requested: item.quantity_requested(),
                quantity_shipped: item.quantity_shipped(),
                quantity_received: item.quantity_received(),
                unit_cost: item.unit_cost(),
                notes: item.notes().map(|s| s.to_string()),
                created_at: item.created_at(),
            })
            .collect();

        TransferDetailResponse {
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
            items,
            created_at: transfer.created_at(),
            updated_at: transfer.updated_at(),
        }
    }
}
