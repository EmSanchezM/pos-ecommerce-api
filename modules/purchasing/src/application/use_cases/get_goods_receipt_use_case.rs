// GetGoodsReceiptUseCase - retrieves a goods receipt by ID

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::responses::{
    GoodsReceiptDetailResponse, GoodsReceiptItemResponse,
};
use crate::domain::entities::GoodsReceipt;
use crate::domain::repositories::GoodsReceiptRepository;
use crate::domain::value_objects::GoodsReceiptId;
use crate::PurchasingError;

/// Use case for retrieving a goods receipt by ID
pub struct GetGoodsReceiptUseCase<G>
where
    G: GoodsReceiptRepository,
{
    receipt_repo: Arc<G>,
}

impl<G> GetGoodsReceiptUseCase<G>
where
    G: GoodsReceiptRepository,
{
    /// Creates a new instance of GetGoodsReceiptUseCase
    pub fn new(receipt_repo: Arc<G>) -> Self {
        Self { receipt_repo }
    }

    /// Executes the use case to retrieve a goods receipt with items
    ///
    /// # Arguments
    /// * `receipt_id` - The ID of the goods receipt to retrieve
    ///
    /// # Returns
    /// GoodsReceiptDetailResponse on success
    pub async fn execute(
        &self,
        receipt_id: Uuid,
    ) -> Result<GoodsReceiptDetailResponse, PurchasingError> {
        let id = GoodsReceiptId::from_uuid(receipt_id);

        let receipt = self
            .receipt_repo
            .find_by_id_with_items(id)
            .await?
            .ok_or(PurchasingError::GoodsReceiptNotFound(receipt_id))?;

        Ok(self.to_detail_response(&receipt))
    }

    fn to_detail_response(&self, receipt: &GoodsReceipt) -> GoodsReceiptDetailResponse {
        let items: Vec<GoodsReceiptItemResponse> = receipt
            .items()
            .iter()
            .map(|item| GoodsReceiptItemResponse {
                id: item.id().into_uuid(),
                goods_receipt_id: item.goods_receipt_id().into_uuid(),
                purchase_order_item_id: item.purchase_order_item_id().into_uuid(),
                product_id: item.product_id().into_uuid(),
                variant_id: item.variant_id().map(|v| v.into_uuid()),
                quantity_received: item.quantity_received(),
                unit_cost: item.unit_cost(),
                lot_number: item.lot_number().map(|s| s.to_string()),
                expiry_date: item.expiry_date(),
                notes: item.notes().map(|s| s.to_string()),
            })
            .collect();

        GoodsReceiptDetailResponse {
            id: receipt.id().into_uuid(),
            receipt_number: receipt.receipt_number().to_string(),
            purchase_order_id: receipt.purchase_order_id().into_uuid(),
            store_id: receipt.store_id().into_uuid(),
            receipt_date: receipt.receipt_date(),
            status: receipt.status().to_string(),
            notes: receipt.notes().map(|s| s.to_string()),
            received_by_id: receipt.received_by_id().into_uuid(),
            confirmed_by_id: receipt.confirmed_by_id().map(|id| id.into_uuid()),
            confirmed_at: receipt.confirmed_at(),
            items,
            created_at: receipt.created_at(),
            updated_at: receipt.updated_at(),
        }
    }
}
