// CreateGoodsReceiptUseCase - creates a new goods receipt for a purchase order

use std::sync::Arc;

use chrono::NaiveDate;

use crate::application::dtos::commands::CreateGoodsReceiptCommand;
use crate::application::dtos::responses::{
    GoodsReceiptDetailResponse, GoodsReceiptItemResponse,
};
use crate::domain::entities::{GoodsReceipt, GoodsReceiptItem};
use crate::domain::repositories::{GoodsReceiptRepository, PurchaseOrderRepository};
use crate::domain::value_objects::{PurchaseOrderId, PurchaseOrderItemId};
use crate::PurchasingError;
use identity::{StoreId, UserId};
use inventory::ProductId;

/// Use case for creating a new goods receipt
pub struct CreateGoodsReceiptUseCase<G, P>
where
    G: GoodsReceiptRepository,
    P: PurchaseOrderRepository,
{
    receipt_repo: Arc<G>,
    order_repo: Arc<P>,
}

impl<G, P> CreateGoodsReceiptUseCase<G, P>
where
    G: GoodsReceiptRepository,
    P: PurchaseOrderRepository,
{
    /// Creates a new instance of CreateGoodsReceiptUseCase
    pub fn new(receipt_repo: Arc<G>, order_repo: Arc<P>) -> Self {
        Self {
            receipt_repo,
            order_repo,
        }
    }

    /// Executes the use case to create a new goods receipt
    ///
    /// # Arguments
    /// * `command` - The create goods receipt command
    /// * `actor_id` - ID of the user creating the receipt
    ///
    /// # Returns
    /// GoodsReceiptDetailResponse on success
    pub async fn execute(
        &self,
        command: CreateGoodsReceiptCommand,
        actor_id: UserId,
    ) -> Result<GoodsReceiptDetailResponse, PurchasingError> {
        // Validate purchase order exists and is approved
        let order_id = PurchaseOrderId::from_uuid(command.purchase_order_id);
        let order = self
            .order_repo
            .find_by_id(order_id)
            .await?
            .ok_or(PurchasingError::PurchaseOrderNotFound(command.purchase_order_id))?;

        // Check order is approved (can receive goods)
        if !order.status().can_receive() {
            return Err(PurchasingError::OrderNotApproved);
        }

        // Parse receipt date
        let receipt_date = NaiveDate::parse_from_str(&command.receipt_date, "%Y-%m-%d")
            .map_err(|_| PurchasingError::InvalidGoodsReceiptStatus)?;

        // Generate receipt number
        let store_id = StoreId::from_uuid(command.store_id);
        let receipt_number = self.receipt_repo.generate_receipt_number(store_id).await?;

        // Create receipt entity
        let mut receipt = GoodsReceipt::create(
            receipt_number,
            order_id,
            store_id,
            receipt_date,
            actor_id,
        );

        // Set notes
        if let Some(notes) = command.notes {
            receipt.set_notes(Some(notes))?;
        }

        // Add items
        for item_cmd in command.items {
            // Parse expiry date if provided
            let expiry_date = item_cmd
                .expiry_date
                .map(|d| {
                    NaiveDate::parse_from_str(&d, "%Y-%m-%d")
                        .map_err(|_| PurchasingError::InvalidGoodsReceiptStatus)
                })
                .transpose()?;

            let mut item = GoodsReceiptItem::create(
                receipt.id(),
                PurchaseOrderItemId::from_uuid(item_cmd.purchase_order_item_id),
                ProductId::from_uuid(item_cmd.product_id),
                item_cmd.variant_id.map(inventory::VariantId::from_uuid),
                item_cmd.quantity_received,
                item_cmd.unit_cost,
            );

            // Set optional fields
            if let Some(lot_number) = item_cmd.lot_number {
                item.set_lot_number(Some(lot_number));
            }
            if let Some(expiry_date) = expiry_date {
                item.set_expiry_date(Some(expiry_date));
            }
            if let Some(notes) = item_cmd.notes {
                item.set_notes(Some(notes));
            }

            receipt.add_item(item)?;
        }

        // Save receipt with items
        self.receipt_repo.save(&receipt).await?;

        // Convert to response
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
