// RejectPurchaseOrderUseCase - rejects a submitted purchase order (returns to draft)

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::commands::RejectOrderCommand;
use crate::application::dtos::responses::{
    PurchaseOrderDetailResponse, PurchaseOrderItemResponse,
};
use crate::domain::entities::PurchaseOrder;
use crate::domain::repositories::PurchaseOrderRepository;
use crate::domain::value_objects::PurchaseOrderId;
use crate::PurchasingError;

/// Use case for rejecting a submitted purchase order
pub struct RejectPurchaseOrderUseCase<P>
where
    P: PurchaseOrderRepository,
{
    order_repo: Arc<P>,
}

impl<P> RejectPurchaseOrderUseCase<P>
where
    P: PurchaseOrderRepository,
{
    /// Creates a new instance of RejectPurchaseOrderUseCase
    pub fn new(order_repo: Arc<P>) -> Self {
        Self { order_repo }
    }

    /// Executes the use case to reject a purchase order
    ///
    /// # Arguments
    /// * `order_id` - The ID of the purchase order to reject
    /// * `command` - The reject command with optional reason
    ///
    /// # Returns
    /// PurchaseOrderDetailResponse on success
    pub async fn execute(
        &self,
        order_id: Uuid,
        command: RejectOrderCommand,
    ) -> Result<PurchaseOrderDetailResponse, PurchasingError> {
        let id = PurchaseOrderId::from_uuid(order_id);

        // Find order with items
        let mut order = self
            .order_repo
            .find_by_id_with_items(id)
            .await?
            .ok_or(PurchasingError::PurchaseOrderNotFound(order_id))?;

        // Reject order (returns to draft)
        order.reject(command.reason)?;

        // Update order
        self.order_repo.update(&order).await?;

        Ok(self.to_detail_response(&order))
    }

    fn to_detail_response(&self, order: &PurchaseOrder) -> PurchaseOrderDetailResponse {
        let items: Vec<PurchaseOrderItemResponse> = order
            .items()
            .iter()
            .map(|item| PurchaseOrderItemResponse {
                id: item.id().into_uuid(),
                purchase_order_id: item.purchase_order_id().into_uuid(),
                line_number: item.line_number(),
                product_id: item.product_id().into_uuid(),
                variant_id: item.variant_id().map(|v| v.into_uuid()),
                description: item.description().to_string(),
                quantity_ordered: item.quantity_ordered(),
                quantity_received: item.quantity_received(),
                unit_of_measure: item.unit_of_measure().to_string(),
                unit_cost: item.unit_cost(),
                discount_percent: item.discount_percent(),
                tax_percent: item.tax_percent(),
                line_total: item.line_total(),
                notes: item.notes().map(|s| s.to_string()),
            })
            .collect();

        PurchaseOrderDetailResponse {
            id: order.id().into_uuid(),
            order_number: order.order_number().to_string(),
            store_id: order.store_id().into_uuid(),
            vendor_id: order.vendor_id().into_uuid(),
            status: order.status().to_string(),
            order_date: order.order_date(),
            expected_delivery_date: order.expected_delivery_date(),
            subtotal: order.subtotal(),
            tax_amount: order.tax_amount(),
            discount_amount: order.discount_amount(),
            total: order.total(),
            currency: order.currency().as_str().to_string(),
            payment_terms_days: order.payment_terms_days(),
            notes: order.notes().map(|s| s.to_string()),
            internal_notes: order.internal_notes().map(|s| s.to_string()),
            created_by_id: order.created_by_id().into_uuid(),
            submitted_by_id: order.submitted_by_id().map(|id| id.into_uuid()),
            submitted_at: order.submitted_at(),
            approved_by_id: order.approved_by_id().map(|id| id.into_uuid()),
            approved_at: order.approved_at(),
            received_by_id: order.received_by_id().map(|id| id.into_uuid()),
            received_date: order.received_date(),
            cancelled_by_id: order.cancelled_by_id().map(|id| id.into_uuid()),
            cancelled_at: order.cancelled_at(),
            cancellation_reason: order.cancellation_reason().map(|s| s.to_string()),
            items,
            created_at: order.created_at(),
            updated_at: order.updated_at(),
        }
    }
}
