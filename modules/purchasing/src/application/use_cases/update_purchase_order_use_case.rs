// UpdatePurchaseOrderUseCase - updates an existing purchase order (draft only)

use std::sync::Arc;

use chrono::NaiveDate;

use crate::PurchasingError;
use crate::application::dtos::commands::UpdatePurchaseOrderCommand;
use crate::application::dtos::responses::{PurchaseOrderDetailResponse, PurchaseOrderItemResponse};
use crate::domain::entities::PurchaseOrder;
use crate::domain::repositories::{PurchaseOrderRepository, VendorRepository};
use crate::domain::value_objects::{PurchaseOrderId, VendorId};

/// Use case for updating an existing purchase order.
///
/// Only draft orders can be updated. Validates vendor existence if changing.
pub struct UpdatePurchaseOrderUseCase<P, V>
where
    P: PurchaseOrderRepository,
    V: VendorRepository,
{
    order_repo: Arc<P>,
    vendor_repo: Arc<V>,
}

impl<P, V> UpdatePurchaseOrderUseCase<P, V>
where
    P: PurchaseOrderRepository,
    V: VendorRepository,
{
    pub fn new(order_repo: Arc<P>, vendor_repo: Arc<V>) -> Self {
        Self {
            order_repo,
            vendor_repo,
        }
    }

    pub async fn execute(
        &self,
        order_id: uuid::Uuid,
        command: UpdatePurchaseOrderCommand,
    ) -> Result<PurchaseOrderDetailResponse, PurchasingError> {
        let po_id = PurchaseOrderId::from_uuid(order_id);
        let mut order = self
            .order_repo
            .find_by_id_with_items(po_id)
            .await?
            .ok_or(PurchasingError::PurchaseOrderNotFound(order_id))?;

        // Validate vendor exists and is active if changing
        if let Some(vendor_uuid) = command.vendor_id {
            let vendor_id = VendorId::from_uuid(vendor_uuid);
            let vendor = self
                .vendor_repo
                .find_by_id(vendor_id)
                .await?
                .ok_or(PurchasingError::VendorNotFound(vendor_uuid))?;
            vendor.validate_active()?;
            order.set_vendor_id(vendor_id)?;
        }

        if let Some(order_date_str) = command.order_date {
            let date = NaiveDate::parse_from_str(&order_date_str, "%Y-%m-%d")
                .map_err(|_| PurchasingError::InvalidPurchaseOrderStatus)?;
            order.set_order_date(date)?;
        }

        if let Some(delivery_date_str) = command.expected_delivery_date {
            let date = NaiveDate::parse_from_str(&delivery_date_str, "%Y-%m-%d")
                .map_err(|_| PurchasingError::InvalidPurchaseOrderStatus)?;
            order.set_expected_delivery_date(Some(date))?;
        }

        if let Some(days) = command.payment_terms_days {
            order.set_payment_terms_days(days)?;
        }

        if let Some(notes) = command.notes {
            order.set_notes(Some(notes))?;
        }

        if let Some(internal_notes) = command.internal_notes {
            order.set_internal_notes(Some(internal_notes))?;
        }

        // Save updated order
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::domain::entities::Vendor;
    use crate::domain::repositories::{PurchaseOrderFilter, VendorFilter};
    use inventory::Currency;

    struct MockOrderRepository {
        orders: Mutex<HashMap<PurchaseOrderId, PurchaseOrder>>,
    }

    impl MockOrderRepository {
        fn new() -> Self {
            Self {
                orders: Mutex::new(HashMap::new()),
            }
        }

        fn add_order(&self, order: PurchaseOrder) {
            let mut orders = self.orders.lock().unwrap();
            orders.insert(order.id(), order);
        }
    }

    #[async_trait]
    impl PurchaseOrderRepository for MockOrderRepository {
        async fn save(&self, order: &PurchaseOrder) -> Result<(), PurchasingError> {
            let mut orders = self.orders.lock().unwrap();
            orders.insert(order.id(), order.clone());
            Ok(())
        }
        async fn find_by_id(
            &self,
            id: PurchaseOrderId,
        ) -> Result<Option<PurchaseOrder>, PurchasingError> {
            let orders = self.orders.lock().unwrap();
            Ok(orders.get(&id).cloned())
        }
        async fn find_by_id_with_items(
            &self,
            id: PurchaseOrderId,
        ) -> Result<Option<PurchaseOrder>, PurchasingError> {
            self.find_by_id(id).await
        }
        async fn find_by_order_number(
            &self,
            _store_id: identity::StoreId,
            _order_number: &str,
        ) -> Result<Option<PurchaseOrder>, PurchasingError> {
            Ok(None)
        }
        async fn update(&self, order: &PurchaseOrder) -> Result<(), PurchasingError> {
            let mut orders = self.orders.lock().unwrap();
            orders.insert(order.id(), order.clone());
            Ok(())
        }
        async fn find_paginated(
            &self,
            _filter: PurchaseOrderFilter,
            _page: i64,
            _page_size: i64,
        ) -> Result<(Vec<PurchaseOrder>, i64), PurchasingError> {
            Ok((vec![], 0))
        }
        async fn generate_order_number(
            &self,
            _store_id: identity::StoreId,
        ) -> Result<String, PurchasingError> {
            Ok("PO-TEST-001".to_string())
        }
        async fn save_item(
            &self,
            _item: &crate::domain::entities::PurchaseOrderItem,
        ) -> Result<(), PurchasingError> {
            Ok(())
        }
        async fn update_item(
            &self,
            _item: &crate::domain::entities::PurchaseOrderItem,
        ) -> Result<(), PurchasingError> {
            Ok(())
        }
        async fn delete_item(
            &self,
            _item_id: crate::domain::value_objects::PurchaseOrderItemId,
        ) -> Result<(), PurchasingError> {
            Ok(())
        }
        async fn find_items_by_order(
            &self,
            _order_id: PurchaseOrderId,
        ) -> Result<Vec<crate::domain::entities::PurchaseOrderItem>, PurchasingError> {
            Ok(vec![])
        }
        async fn find_item_by_id(
            &self,
            _item_id: crate::domain::value_objects::PurchaseOrderItemId,
        ) -> Result<Option<crate::domain::entities::PurchaseOrderItem>, PurchasingError> {
            Ok(None)
        }
    }

    struct MockVendorRepository {
        vendors: Mutex<HashMap<VendorId, Vendor>>,
    }

    impl MockVendorRepository {
        fn new() -> Self {
            Self {
                vendors: Mutex::new(HashMap::new()),
            }
        }

        fn add_vendor(&self, vendor: Vendor) {
            let mut vendors = self.vendors.lock().unwrap();
            vendors.insert(vendor.id(), vendor);
        }
    }

    #[async_trait]
    impl VendorRepository for MockVendorRepository {
        async fn save(&self, vendor: &Vendor) -> Result<(), PurchasingError> {
            let mut vendors = self.vendors.lock().unwrap();
            vendors.insert(vendor.id(), vendor.clone());
            Ok(())
        }
        async fn find_by_id(&self, id: VendorId) -> Result<Option<Vendor>, PurchasingError> {
            let vendors = self.vendors.lock().unwrap();
            Ok(vendors.get(&id).cloned())
        }
        async fn find_by_code(&self, _code: &str) -> Result<Option<Vendor>, PurchasingError> {
            Ok(None)
        }
        async fn update(&self, _vendor: &Vendor) -> Result<(), PurchasingError> {
            Ok(())
        }
        async fn find_paginated(
            &self,
            _filter: VendorFilter,
            _page: i64,
            _page_size: i64,
        ) -> Result<(Vec<Vendor>, i64), PurchasingError> {
            Ok((vec![], 0))
        }
        async fn count(&self, _filter: VendorFilter) -> Result<i64, PurchasingError> {
            Ok(0)
        }
        async fn exists_by_code(&self, _code: &str) -> Result<bool, PurchasingError> {
            Ok(false)
        }
        async fn exists_by_tax_id(&self, _tax_id: &str) -> Result<bool, PurchasingError> {
            Ok(false)
        }
        async fn exists_by_code_excluding(
            &self,
            _code: &str,
            _exclude_id: VendorId,
        ) -> Result<bool, PurchasingError> {
            Ok(false)
        }
        async fn exists_by_tax_id_excluding(
            &self,
            _tax_id: &str,
            _exclude_id: VendorId,
        ) -> Result<bool, PurchasingError> {
            Ok(false)
        }
        async fn count_by_code_prefix(&self, _prefix: &str) -> Result<i64, PurchasingError> {
            Ok(0)
        }
    }

    fn create_test_order(vendor_id: VendorId) -> PurchaseOrder {
        use chrono::NaiveDate;
        use identity::{StoreId, UserId};

        PurchaseOrder::create(
            "PO-TEST-001".to_string(),
            StoreId::new(),
            vendor_id,
            NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            Currency::hnl(),
            30,
            UserId::new(),
        )
    }

    fn create_test_vendor() -> Vendor {
        Vendor::create(
            "V-TEST-001".to_string(),
            "Test Vendor".to_string(),
            "Test Vendor SRL".to_string(),
            "0801-1990-00001".to_string(),
            Currency::hnl(),
        )
    }

    #[tokio::test]
    async fn test_update_purchase_order_notes() {
        let order_repo = Arc::new(MockOrderRepository::new());
        let vendor_repo = Arc::new(MockVendorRepository::new());

        let vendor = create_test_vendor();
        vendor_repo.add_vendor(vendor.clone());

        let order = create_test_order(vendor.id());
        let order_id = order.id().into_uuid();
        order_repo.add_order(order);

        let use_case = UpdatePurchaseOrderUseCase::new(order_repo, vendor_repo);

        let command = UpdatePurchaseOrderCommand {
            vendor_id: None,
            order_date: None,
            expected_delivery_date: None,
            payment_terms_days: Some(60),
            notes: Some("Updated notes".to_string()),
            internal_notes: None,
        };

        let result = use_case.execute(order_id, command).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.payment_terms_days, 60);
        assert_eq!(response.notes, Some("Updated notes".to_string()));
    }

    #[tokio::test]
    async fn test_update_purchase_order_not_found() {
        let order_repo = Arc::new(MockOrderRepository::new());
        let vendor_repo = Arc::new(MockVendorRepository::new());

        let use_case = UpdatePurchaseOrderUseCase::new(order_repo, vendor_repo);

        let command = UpdatePurchaseOrderCommand {
            vendor_id: None,
            order_date: None,
            expected_delivery_date: None,
            payment_terms_days: None,
            notes: Some("test".to_string()),
            internal_notes: None,
        };

        let result = use_case
            .execute(PurchaseOrderId::new().into_uuid(), command)
            .await;
        assert!(matches!(
            result,
            Err(PurchasingError::PurchaseOrderNotFound(_))
        ));
    }
}
