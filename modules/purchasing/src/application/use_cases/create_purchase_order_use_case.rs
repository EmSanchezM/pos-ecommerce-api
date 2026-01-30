// CreatePurchaseOrderUseCase - creates a new purchase order

use std::str::FromStr;
use std::sync::Arc;

use chrono::NaiveDate;

use crate::application::dtos::commands::CreatePurchaseOrderCommand;
use crate::application::dtos::responses::{
    PurchaseOrderDetailResponse, PurchaseOrderItemResponse,
};
use crate::domain::entities::{PurchaseOrder, PurchaseOrderItem};
use crate::domain::repositories::{PurchaseOrderRepository, VendorRepository};
use crate::domain::value_objects::VendorId;
use crate::PurchasingError;
use identity::{StoreId, UserId};
use inventory::{Currency, ProductId, UnitOfMeasure};

/// Use case for creating a new purchase order
pub struct CreatePurchaseOrderUseCase<P, V>
where
    P: PurchaseOrderRepository,
    V: VendorRepository,
{
    order_repo: Arc<P>,
    vendor_repo: Arc<V>,
}

impl<P, V> CreatePurchaseOrderUseCase<P, V>
where
    P: PurchaseOrderRepository,
    V: VendorRepository,
{
    /// Creates a new instance of CreatePurchaseOrderUseCase
    pub fn new(order_repo: Arc<P>, vendor_repo: Arc<V>) -> Self {
        Self {
            order_repo,
            vendor_repo,
        }
    }

    /// Executes the use case to create a new purchase order
    ///
    /// # Arguments
    /// * `command` - The create purchase order command
    /// * `actor_id` - ID of the user creating the order
    ///
    /// # Returns
    /// PurchaseOrderDetailResponse on success
    pub async fn execute(
        &self,
        command: CreatePurchaseOrderCommand,
        actor_id: UserId,
    ) -> Result<PurchaseOrderDetailResponse, PurchasingError> {
        // Validate vendor exists and is active
        let vendor_id = VendorId::from_uuid(command.vendor_id);
        let vendor = self
            .vendor_repo
            .find_by_id(vendor_id)
            .await?
            .ok_or(PurchasingError::VendorNotFound(command.vendor_id))?;
        vendor.validate_active()?;

        // Parse dates
        let order_date = NaiveDate::parse_from_str(&command.order_date, "%Y-%m-%d")
            .map_err(|_| PurchasingError::InvalidPurchaseOrderStatus)?;
        let expected_delivery_date = command
            .expected_delivery_date
            .map(|d| {
                NaiveDate::parse_from_str(&d, "%Y-%m-%d")
                    .map_err(|_| PurchasingError::InvalidPurchaseOrderStatus)
            })
            .transpose()?;

        // Determine currency (use command value, or vendor's default)
        let currency = if let Some(ref currency_str) = command.currency {
            Currency::new(currency_str)
                .map_err(|_| PurchasingError::InvalidCurrency)?
        } else {
            vendor.currency().clone()
        };

        // Determine payment terms (use command value, or vendor's default)
        let payment_terms_days = command
            .payment_terms_days
            .unwrap_or(vendor.payment_terms_days());

        // Generate order number
        let store_id = StoreId::from_uuid(command.store_id);
        let order_number = self.order_repo.generate_order_number(store_id).await?;

        // Create order entity
        let mut order = PurchaseOrder::create(
            order_number,
            store_id,
            vendor_id,
            order_date,
            currency,
            payment_terms_days,
            actor_id,
        );

        // Set expected delivery date
        if let Some(date) = expected_delivery_date {
            order.set_expected_delivery_date(Some(date))?;
        }

        // Set notes
        if let Some(notes) = command.notes {
            order.set_notes(Some(notes))?;
        }

        // Add items
        for (index, item_cmd) in command.items.into_iter().enumerate() {
            let unit_of_measure = UnitOfMeasure::from_str(&item_cmd.unit_of_measure)
                .map_err(|_| PurchasingError::InvalidUnitOfMeasure)?;

            let item = PurchaseOrderItem::create(
                order.id(),
                (index + 1) as i32,
                ProductId::from_uuid(item_cmd.product_id),
                item_cmd.variant_id.map(inventory::VariantId::from_uuid),
                item_cmd.description,
                item_cmd.quantity_ordered,
                unit_of_measure,
                item_cmd.unit_cost,
                item_cmd.discount_percent,
                item_cmd.tax_percent,
            );

            order.add_item(item)?;
        }

        // Save order with items
        self.order_repo.save(&order).await?;

        // Convert to response
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
