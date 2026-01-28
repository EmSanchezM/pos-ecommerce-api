// ConfirmGoodsReceiptUseCase - confirms a goods receipt and updates inventory

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::responses::{
    GoodsReceiptDetailResponse, GoodsReceiptItemResponse,
};
use crate::domain::entities::GoodsReceipt;
use crate::domain::repositories::{GoodsReceiptRepository, PurchaseOrderRepository};
use crate::domain::value_objects::GoodsReceiptId;
use crate::PurchasingError;
use identity::UserId;

/// Use case for confirming a goods receipt
pub struct ConfirmGoodsReceiptUseCase<G, P>
where
    G: GoodsReceiptRepository,
    P: PurchaseOrderRepository,
{
    receipt_repo: Arc<G>,
    order_repo: Arc<P>,
}

impl<G, P> ConfirmGoodsReceiptUseCase<G, P>
where
    G: GoodsReceiptRepository,
    P: PurchaseOrderRepository,
{
    /// Creates a new instance of ConfirmGoodsReceiptUseCase
    pub fn new(receipt_repo: Arc<G>, order_repo: Arc<P>) -> Self {
        Self {
            receipt_repo,
            order_repo,
        }
    }

    /// Executes the use case to confirm a goods receipt
    ///
    /// # Arguments
    /// * `receipt_id` - The ID of the goods receipt to confirm
    /// * `actor_id` - ID of the user confirming the receipt
    ///
    /// # Returns
    /// GoodsReceiptDetailResponse on success
    pub async fn execute(
        &self,
        receipt_id: Uuid,
        actor_id: UserId,
    ) -> Result<GoodsReceiptDetailResponse, PurchasingError> {
        let id = GoodsReceiptId::from_uuid(receipt_id);

        // Find receipt with items
        let mut receipt = self
            .receipt_repo
            .find_by_id_with_items(id)
            .await?
            .ok_or(PurchasingError::GoodsReceiptNotFound(receipt_id))?;

        // Confirm receipt
        receipt.confirm(actor_id)?;

        // Update receipt
        self.receipt_repo.update(&receipt).await?;

        // Update purchase order received quantities
        let mut order = self
            .order_repo
            .find_by_id_with_items(receipt.purchase_order_id())
            .await?
            .ok_or(PurchasingError::PurchaseOrderNotFound(
                receipt.purchase_order_id().into_uuid(),
            ))?;

        // Update received quantities for matching order items
        for receipt_item in receipt.items() {
            for order_item in order.items_mut() {
                if order_item.id() == receipt_item.purchase_order_item_id() {
                    order_item.add_received_quantity(receipt_item.quantity_received());
                    break;
                }
            }
        }

        // Update order status based on received quantities
        if order.all_items_received() {
            order.receive_complete(actor_id, receipt.receipt_date())?;
        } else if order.has_received_items() {
            order.receive_partial(actor_id)?;
        }

        // Update order
        self.order_repo.update(&order).await?;

        // Note: In a real implementation, this would also update inventory stock
        // using an InventoryRepository or event-driven approach

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
