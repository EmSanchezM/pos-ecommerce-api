// PurchaseOrder entity - document for ordering goods from vendors

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::entities::PurchaseOrderItem;
use crate::domain::value_objects::{PurchaseOrderId, PurchaseOrderStatus, VendorId};
use crate::PurchasingError;
use identity::{StoreId, UserId};
use inventory::Currency;

/// PurchaseOrder entity representing a document for ordering goods from vendors.
/// Implements an approval workflow: draft → submitted → approved → received → closed
///
/// Invariants:
/// - Status transitions must follow the defined workflow
/// - Only draft orders can be modified
/// - User cannot approve their own order
/// - Must have items before submitting for approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    id: PurchaseOrderId,
    order_number: String,
    store_id: StoreId,
    vendor_id: VendorId,
    status: PurchaseOrderStatus,
    order_date: NaiveDate,
    expected_delivery_date: Option<NaiveDate>,
    received_date: Option<NaiveDate>,
    subtotal: Decimal,
    tax_amount: Decimal,
    discount_amount: Decimal,
    total: Decimal,
    currency: Currency,
    payment_terms_days: i32,
    notes: Option<String>,
    internal_notes: Option<String>,
    created_by_id: UserId,
    submitted_by_id: Option<UserId>,
    submitted_at: Option<DateTime<Utc>>,
    approved_by_id: Option<UserId>,
    approved_at: Option<DateTime<Utc>>,
    received_by_id: Option<UserId>,
    cancelled_by_id: Option<UserId>,
    cancelled_at: Option<DateTime<Utc>>,
    cancellation_reason: Option<String>,
    items: Vec<PurchaseOrderItem>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl PurchaseOrder {
    /// Creates a new PurchaseOrder in draft status
    pub fn create(
        order_number: String,
        store_id: StoreId,
        vendor_id: VendorId,
        order_date: NaiveDate,
        currency: Currency,
        payment_terms_days: i32,
        created_by_id: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: PurchaseOrderId::new(),
            order_number,
            store_id,
            vendor_id,
            status: PurchaseOrderStatus::Draft,
            order_date,
            expected_delivery_date: None,
            received_date: None,
            subtotal: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            discount_amount: Decimal::ZERO,
            total: Decimal::ZERO,
            currency,
            payment_terms_days,
            notes: None,
            internal_notes: None,
            created_by_id,
            submitted_by_id: None,
            submitted_at: None,
            approved_by_id: None,
            approved_at: None,
            received_by_id: None,
            cancelled_by_id: None,
            cancelled_at: None,
            cancellation_reason: None,
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a PurchaseOrder from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: PurchaseOrderId,
        order_number: String,
        store_id: StoreId,
        vendor_id: VendorId,
        status: PurchaseOrderStatus,
        order_date: NaiveDate,
        expected_delivery_date: Option<NaiveDate>,
        received_date: Option<NaiveDate>,
        subtotal: Decimal,
        tax_amount: Decimal,
        discount_amount: Decimal,
        total: Decimal,
        currency: Currency,
        payment_terms_days: i32,
        notes: Option<String>,
        internal_notes: Option<String>,
        created_by_id: UserId,
        submitted_by_id: Option<UserId>,
        submitted_at: Option<DateTime<Utc>>,
        approved_by_id: Option<UserId>,
        approved_at: Option<DateTime<Utc>>,
        received_by_id: Option<UserId>,
        cancelled_by_id: Option<UserId>,
        cancelled_at: Option<DateTime<Utc>>,
        cancellation_reason: Option<String>,
        items: Vec<PurchaseOrderItem>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            order_number,
            store_id,
            vendor_id,
            status,
            order_date,
            expected_delivery_date,
            received_date,
            subtotal,
            tax_amount,
            discount_amount,
            total,
            currency,
            payment_terms_days,
            notes,
            internal_notes,
            created_by_id,
            submitted_by_id,
            submitted_at,
            approved_by_id,
            approved_at,
            received_by_id,
            cancelled_by_id,
            cancelled_at,
            cancellation_reason,
            items,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Workflow Methods
    // =========================================================================

    /// Submits the order for approval
    /// Transitions: draft → submitted
    pub fn submit(&mut self, submitted_by_id: UserId) -> Result<(), PurchasingError> {
        if !self.status.can_submit() {
            return Err(PurchasingError::InvalidStatusTransition);
        }
        if self.items.is_empty() {
            return Err(PurchasingError::EmptyPurchaseOrder);
        }

        self.status = PurchaseOrderStatus::Submitted;
        self.submitted_by_id = Some(submitted_by_id);
        self.submitted_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Approves the order
    /// Transitions: submitted → approved
    pub fn approve(&mut self, approver_id: UserId) -> Result<(), PurchasingError> {
        if !self.status.can_review() {
            return Err(PurchasingError::InvalidStatusTransition);
        }
        // User cannot approve their own order
        if approver_id == self.created_by_id {
            return Err(PurchasingError::CannotApproveSelfCreatedOrder);
        }

        self.status = PurchaseOrderStatus::Approved;
        self.approved_by_id = Some(approver_id);
        self.approved_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Rejects the order, returning it to draft
    /// Transitions: submitted → draft
    pub fn reject(&mut self, reason: Option<String>) -> Result<(), PurchasingError> {
        if !self.status.can_review() {
            return Err(PurchasingError::InvalidStatusTransition);
        }

        self.status = PurchaseOrderStatus::Draft;
        self.submitted_by_id = None;
        self.submitted_at = None;
        self.internal_notes = reason;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Marks the order as partially received
    /// Transitions: approved → partially_received
    pub fn receive_partial(&mut self, received_by_id: UserId) -> Result<(), PurchasingError> {
        if !self.status.can_receive() {
            return Err(PurchasingError::InvalidStatusTransition);
        }

        self.status = PurchaseOrderStatus::PartiallyReceived;
        self.received_by_id = Some(received_by_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Marks the order as completely received
    /// Transitions: approved/partially_received → received
    pub fn receive_complete(
        &mut self,
        received_by_id: UserId,
        received_date: NaiveDate,
    ) -> Result<(), PurchasingError> {
        if !self.status.can_receive() {
            return Err(PurchasingError::InvalidStatusTransition);
        }

        self.status = PurchaseOrderStatus::Received;
        self.received_by_id = Some(received_by_id);
        self.received_date = Some(received_date);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Closes the order
    /// Transitions: received → closed
    pub fn close(&mut self) -> Result<(), PurchasingError> {
        if !self.status.can_close() {
            return Err(PurchasingError::InvalidStatusTransition);
        }

        self.status = PurchaseOrderStatus::Closed;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancels the order
    /// Transitions: draft/submitted → cancelled
    pub fn cancel(
        &mut self,
        cancelled_by_id: UserId,
        reason: String,
    ) -> Result<(), PurchasingError> {
        if !self.status.can_cancel() {
            return Err(PurchasingError::InvalidStatusTransition);
        }

        self.status = PurchaseOrderStatus::Cancelled;
        self.cancelled_by_id = Some(cancelled_by_id);
        self.cancelled_at = Some(Utc::now());
        self.cancellation_reason = Some(reason);
        self.updated_at = Utc::now();
        Ok(())
    }

    // =========================================================================
    // Item Management
    // =========================================================================

    /// Adds an item to the order (only allowed in draft status)
    pub fn add_item(&mut self, item: PurchaseOrderItem) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::OrderNotEditable);
        }
        self.items.push(item);
        self.recalculate_totals();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Removes an item from the order (only allowed in draft status)
    pub fn remove_item(
        &mut self,
        item_id: crate::domain::value_objects::PurchaseOrderItemId,
    ) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::OrderNotEditable);
        }
        self.items.retain(|i| i.id() != item_id);
        self.recalculate_totals();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Recalculates the order totals from items
    pub fn recalculate_totals(&mut self) {
        let mut subtotal = Decimal::ZERO;
        let mut tax_amount = Decimal::ZERO;
        let mut discount_amount = Decimal::ZERO;

        for item in &self.items {
            let item_subtotal = item.quantity_ordered() * item.unit_cost();
            let item_discount =
                item_subtotal * (item.discount_percent() / Decimal::from(100));
            let after_discount = item_subtotal - item_discount;
            let item_tax = after_discount * (item.tax_percent() / Decimal::from(100));

            subtotal += item_subtotal;
            discount_amount += item_discount;
            tax_amount += item_tax;
        }

        self.subtotal = subtotal;
        self.discount_amount = discount_amount;
        self.tax_amount = tax_amount;
        self.total = subtotal - discount_amount + tax_amount;
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Returns true if the order can be edited
    pub fn is_editable(&self) -> bool {
        self.status.is_editable()
    }

    /// Returns true if the order is in a final state
    pub fn is_final(&self) -> bool {
        self.status.is_final()
    }

    /// Checks if all items are fully received
    pub fn all_items_received(&self) -> bool {
        self.items.iter().all(|item| item.is_fully_received())
    }

    /// Checks if any items have been received
    pub fn has_received_items(&self) -> bool {
        self.items
            .iter()
            .any(|item| item.quantity_received() > Decimal::ZERO)
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> PurchaseOrderId {
        self.id
    }

    pub fn order_number(&self) -> &str {
        &self.order_number
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn status(&self) -> PurchaseOrderStatus {
        self.status
    }

    pub fn order_date(&self) -> NaiveDate {
        self.order_date
    }

    pub fn expected_delivery_date(&self) -> Option<NaiveDate> {
        self.expected_delivery_date
    }

    pub fn received_date(&self) -> Option<NaiveDate> {
        self.received_date
    }

    pub fn subtotal(&self) -> Decimal {
        self.subtotal
    }

    pub fn tax_amount(&self) -> Decimal {
        self.tax_amount
    }

    pub fn discount_amount(&self) -> Decimal {
        self.discount_amount
    }

    pub fn total(&self) -> Decimal {
        self.total
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn payment_terms_days(&self) -> i32 {
        self.payment_terms_days
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn internal_notes(&self) -> Option<&str> {
        self.internal_notes.as_deref()
    }

    pub fn created_by_id(&self) -> UserId {
        self.created_by_id
    }

    pub fn submitted_by_id(&self) -> Option<UserId> {
        self.submitted_by_id
    }

    pub fn submitted_at(&self) -> Option<DateTime<Utc>> {
        self.submitted_at
    }

    pub fn approved_by_id(&self) -> Option<UserId> {
        self.approved_by_id
    }

    pub fn approved_at(&self) -> Option<DateTime<Utc>> {
        self.approved_at
    }

    pub fn received_by_id(&self) -> Option<UserId> {
        self.received_by_id
    }

    pub fn cancelled_by_id(&self) -> Option<UserId> {
        self.cancelled_by_id
    }

    pub fn cancelled_at(&self) -> Option<DateTime<Utc>> {
        self.cancelled_at
    }

    pub fn cancellation_reason(&self) -> Option<&str> {
        self.cancellation_reason.as_deref()
    }

    pub fn items(&self) -> &[PurchaseOrderItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<PurchaseOrderItem> {
        &mut self.items
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // =========================================================================
    // Setters (only allowed in draft status)
    // =========================================================================

    pub fn set_vendor_id(&mut self, vendor_id: VendorId) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::OrderNotEditable);
        }
        self.vendor_id = vendor_id;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_order_date(&mut self, date: NaiveDate) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::OrderNotEditable);
        }
        self.order_date = date;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_expected_delivery_date(
        &mut self,
        date: Option<NaiveDate>,
    ) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::OrderNotEditable);
        }
        self.expected_delivery_date = date;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_payment_terms_days(&mut self, days: i32) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::OrderNotEditable);
        }
        self.payment_terms_days = days;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_notes(&mut self, notes: Option<String>) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::OrderNotEditable);
        }
        self.notes = notes;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_internal_notes(
        &mut self,
        internal_notes: Option<String>,
    ) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::OrderNotEditable);
        }
        self.internal_notes = internal_notes;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    fn create_test_order() -> PurchaseOrder {
        PurchaseOrder::create(
            "PO-2024-00001".to_string(),
            StoreId::new(),
            VendorId::new(),
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            Currency::new("HNL").unwrap(),
            30,
            UserId::new(),
        )
    }

    fn create_test_item(order_id: PurchaseOrderId) -> PurchaseOrderItem {
        use inventory::{ProductId, UnitOfMeasure};
        PurchaseOrderItem::create(
            order_id,
            1,
            ProductId::new(),
            None,
            "Test Product".to_string(),
            dec!(10),
            UnitOfMeasure::from_str("unit").unwrap(),
            dec!(100.00),
            dec!(0),
            dec!(15),
        )
    }

    #[test]
    fn test_create_order() {
        let order = create_test_order();

        assert_eq!(order.order_number(), "PO-2024-00001");
        assert_eq!(order.status(), PurchaseOrderStatus::Draft);
        assert!(order.is_editable());
        assert!(!order.is_final());
        assert!(order.items().is_empty());
    }

    #[test]
    fn test_add_item() {
        let mut order = create_test_order();
        let item = create_test_item(order.id());

        order.add_item(item).unwrap();

        assert_eq!(order.items().len(), 1);
        assert!(order.total() > Decimal::ZERO);
    }

    #[test]
    fn test_submit_workflow() {
        let mut order = create_test_order();
        let item = create_test_item(order.id());
        order.add_item(item).unwrap();

        let submitter = UserId::new();
        order.submit(submitter).unwrap();

        assert_eq!(order.status(), PurchaseOrderStatus::Submitted);
        assert_eq!(order.submitted_by_id(), Some(submitter));
        assert!(order.submitted_at().is_some());
        assert!(!order.is_editable());
    }

    #[test]
    fn test_submit_empty_order() {
        let mut order = create_test_order();

        let result = order.submit(UserId::new());

        assert!(matches!(result, Err(PurchasingError::EmptyPurchaseOrder)));
    }

    #[test]
    fn test_approve_workflow() {
        let mut order = create_test_order();
        let item = create_test_item(order.id());
        order.add_item(item).unwrap();
        order.submit(UserId::new()).unwrap();

        let approver = UserId::new(); // Different user
        order.approve(approver).unwrap();

        assert_eq!(order.status(), PurchaseOrderStatus::Approved);
        assert_eq!(order.approved_by_id(), Some(approver));
        assert!(order.approved_at().is_some());
    }

    #[test]
    fn test_cannot_approve_own_order() {
        let creator = UserId::new();
        let mut order = PurchaseOrder::create(
            "PO-2024-00001".to_string(),
            StoreId::new(),
            VendorId::new(),
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            Currency::new("HNL").unwrap(),
            30,
            creator,
        );
        let item = create_test_item(order.id());
        order.add_item(item).unwrap();
        order.submit(creator).unwrap();

        let result = order.approve(creator); // Same user as creator

        assert!(matches!(
            result,
            Err(PurchasingError::CannotApproveSelfCreatedOrder)
        ));
    }

    #[test]
    fn test_reject_workflow() {
        let mut order = create_test_order();
        let item = create_test_item(order.id());
        order.add_item(item).unwrap();
        order.submit(UserId::new()).unwrap();

        order.reject(Some("Invalid quantities".to_string())).unwrap();

        assert_eq!(order.status(), PurchaseOrderStatus::Draft);
        assert!(order.is_editable());
        assert_eq!(order.internal_notes(), Some("Invalid quantities"));
    }

    #[test]
    fn test_cancel_workflow() {
        let mut order = create_test_order();
        let item = create_test_item(order.id());
        order.add_item(item).unwrap();

        let canceller = UserId::new();
        order
            .cancel(canceller, "No longer needed".to_string())
            .unwrap();

        assert_eq!(order.status(), PurchaseOrderStatus::Cancelled);
        assert_eq!(order.cancelled_by_id(), Some(canceller));
        assert_eq!(order.cancellation_reason(), Some("No longer needed"));
        assert!(order.is_final());
    }

    #[test]
    fn test_receive_workflow() {
        let mut order = create_test_order();
        let item = create_test_item(order.id());
        order.add_item(item).unwrap();
        order.submit(UserId::new()).unwrap();
        order.approve(UserId::new()).unwrap();

        let receiver = UserId::new();
        order.receive_partial(receiver).unwrap();
        assert_eq!(order.status(), PurchaseOrderStatus::PartiallyReceived);

        order
            .receive_complete(receiver, NaiveDate::from_ymd_opt(2024, 1, 25).unwrap())
            .unwrap();
        assert_eq!(order.status(), PurchaseOrderStatus::Received);
        assert!(order.received_date().is_some());
    }

    #[test]
    fn test_close_workflow() {
        let mut order = create_test_order();
        let item = create_test_item(order.id());
        order.add_item(item).unwrap();
        order.submit(UserId::new()).unwrap();
        order.approve(UserId::new()).unwrap();
        order
            .receive_complete(UserId::new(), NaiveDate::from_ymd_opt(2024, 1, 25).unwrap())
            .unwrap();

        order.close().unwrap();

        assert_eq!(order.status(), PurchaseOrderStatus::Closed);
        assert!(order.is_final());
    }

    #[test]
    fn test_cannot_modify_submitted_order() {
        let mut order = create_test_order();
        let item = create_test_item(order.id());
        order.add_item(item).unwrap();
        order.submit(UserId::new()).unwrap();

        let result = order.set_notes(Some("New notes".to_string()));

        assert!(matches!(result, Err(PurchasingError::OrderNotEditable)));
    }
}
