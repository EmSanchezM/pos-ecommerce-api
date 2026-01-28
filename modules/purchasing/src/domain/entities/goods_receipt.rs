// GoodsReceipt entity - document for receiving goods from a purchase order

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::entities::GoodsReceiptItem;
use crate::domain::value_objects::{GoodsReceiptId, GoodsReceiptStatus, PurchaseOrderId};
use crate::PurchasingError;
use identity::{StoreId, UserId};

/// GoodsReceipt entity representing a document for receiving goods from a purchase order.
/// Implements a simple workflow: draft → confirmed/cancelled
///
/// Invariants:
/// - Status transitions must follow the defined workflow
/// - Only draft receipts can be modified
/// - Must have items before confirming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceipt {
    id: GoodsReceiptId,
    receipt_number: String,
    purchase_order_id: PurchaseOrderId,
    store_id: StoreId,
    receipt_date: NaiveDate,
    status: GoodsReceiptStatus,
    notes: Option<String>,
    received_by_id: UserId,
    confirmed_by_id: Option<UserId>,
    confirmed_at: Option<DateTime<Utc>>,
    items: Vec<GoodsReceiptItem>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl GoodsReceipt {
    /// Creates a new GoodsReceipt in draft status
    pub fn create(
        receipt_number: String,
        purchase_order_id: PurchaseOrderId,
        store_id: StoreId,
        receipt_date: NaiveDate,
        received_by_id: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: GoodsReceiptId::new(),
            receipt_number,
            purchase_order_id,
            store_id,
            receipt_date,
            status: GoodsReceiptStatus::Draft,
            notes: None,
            received_by_id,
            confirmed_by_id: None,
            confirmed_at: None,
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a GoodsReceipt from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: GoodsReceiptId,
        receipt_number: String,
        purchase_order_id: PurchaseOrderId,
        store_id: StoreId,
        receipt_date: NaiveDate,
        status: GoodsReceiptStatus,
        notes: Option<String>,
        received_by_id: UserId,
        confirmed_by_id: Option<UserId>,
        confirmed_at: Option<DateTime<Utc>>,
        items: Vec<GoodsReceiptItem>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            receipt_number,
            purchase_order_id,
            store_id,
            receipt_date,
            status,
            notes,
            received_by_id,
            confirmed_by_id,
            confirmed_at,
            items,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Workflow Methods
    // =========================================================================

    /// Confirms the receipt (applies to inventory)
    /// Transitions: draft → confirmed
    pub fn confirm(&mut self, confirmed_by_id: UserId) -> Result<(), PurchasingError> {
        if !self.status.can_confirm() {
            return Err(PurchasingError::InvalidStatusTransition);
        }
        if self.items.is_empty() {
            return Err(PurchasingError::EmptyGoodsReceipt);
        }

        self.status = GoodsReceiptStatus::Confirmed;
        self.confirmed_by_id = Some(confirmed_by_id);
        self.confirmed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancels the receipt
    /// Transitions: draft → cancelled
    pub fn cancel(&mut self) -> Result<(), PurchasingError> {
        if !self.status.can_cancel() {
            return Err(PurchasingError::InvalidStatusTransition);
        }

        self.status = GoodsReceiptStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    // =========================================================================
    // Item Management
    // =========================================================================

    /// Adds an item to the receipt (only allowed in draft status)
    pub fn add_item(&mut self, item: GoodsReceiptItem) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::ReceiptNotEditable);
        }
        self.items.push(item);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Removes an item from the receipt (only allowed in draft status)
    pub fn remove_item(
        &mut self,
        item_id: crate::domain::value_objects::GoodsReceiptItemId,
    ) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::ReceiptNotEditable);
        }
        self.items.retain(|i| i.id() != item_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Returns true if the receipt can be edited
    pub fn is_editable(&self) -> bool {
        self.status.is_editable()
    }

    /// Returns true if the receipt is in a final state
    pub fn is_final(&self) -> bool {
        self.status.is_final()
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> GoodsReceiptId {
        self.id
    }

    pub fn receipt_number(&self) -> &str {
        &self.receipt_number
    }

    pub fn purchase_order_id(&self) -> PurchaseOrderId {
        self.purchase_order_id
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn receipt_date(&self) -> NaiveDate {
        self.receipt_date
    }

    pub fn status(&self) -> GoodsReceiptStatus {
        self.status
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn received_by_id(&self) -> UserId {
        self.received_by_id
    }

    pub fn confirmed_by_id(&self) -> Option<UserId> {
        self.confirmed_by_id
    }

    pub fn confirmed_at(&self) -> Option<DateTime<Utc>> {
        self.confirmed_at
    }

    pub fn items(&self) -> &[GoodsReceiptItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<GoodsReceiptItem> {
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

    pub fn set_receipt_date(&mut self, date: NaiveDate) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::ReceiptNotEditable);
        }
        self.receipt_date = date;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_notes(&mut self, notes: Option<String>) -> Result<(), PurchasingError> {
        if !self.is_editable() {
            return Err(PurchasingError::ReceiptNotEditable);
        }
        self.notes = notes;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_receipt() -> GoodsReceipt {
        GoodsReceipt::create(
            "GR-2024-00001".to_string(),
            PurchaseOrderId::new(),
            StoreId::new(),
            NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
            UserId::new(),
        )
    }

    fn create_test_item(receipt_id: GoodsReceiptId) -> GoodsReceiptItem {
        use crate::domain::value_objects::PurchaseOrderItemId;
        use inventory::ProductId;
        GoodsReceiptItem::create(
            receipt_id,
            PurchaseOrderItemId::new(),
            ProductId::new(),
            None,
            dec!(10),
            dec!(100.00),
        )
    }

    #[test]
    fn test_create_receipt() {
        let receipt = create_test_receipt();

        assert_eq!(receipt.receipt_number(), "GR-2024-00001");
        assert_eq!(receipt.status(), GoodsReceiptStatus::Draft);
        assert!(receipt.is_editable());
        assert!(!receipt.is_final());
        assert!(receipt.items().is_empty());
    }

    #[test]
    fn test_add_item() {
        let mut receipt = create_test_receipt();
        let item = create_test_item(receipt.id());

        receipt.add_item(item).unwrap();

        assert_eq!(receipt.items().len(), 1);
    }

    #[test]
    fn test_confirm_workflow() {
        let mut receipt = create_test_receipt();
        let item = create_test_item(receipt.id());
        receipt.add_item(item).unwrap();

        let confirmer = UserId::new();
        receipt.confirm(confirmer).unwrap();

        assert_eq!(receipt.status(), GoodsReceiptStatus::Confirmed);
        assert_eq!(receipt.confirmed_by_id(), Some(confirmer));
        assert!(receipt.confirmed_at().is_some());
        assert!(receipt.is_final());
        assert!(!receipt.is_editable());
    }

    #[test]
    fn test_confirm_empty_receipt() {
        let mut receipt = create_test_receipt();

        let result = receipt.confirm(UserId::new());

        assert!(matches!(result, Err(PurchasingError::EmptyGoodsReceipt)));
    }

    #[test]
    fn test_cancel_workflow() {
        let mut receipt = create_test_receipt();

        receipt.cancel().unwrap();

        assert_eq!(receipt.status(), GoodsReceiptStatus::Cancelled);
        assert!(receipt.is_final());
    }

    #[test]
    fn test_cannot_modify_confirmed_receipt() {
        let mut receipt = create_test_receipt();
        let item = create_test_item(receipt.id());
        receipt.add_item(item).unwrap();
        receipt.confirm(UserId::new()).unwrap();

        let result = receipt.set_notes(Some("New notes".to_string()));

        assert!(matches!(result, Err(PurchasingError::ReceiptNotEditable)));
    }

    #[test]
    fn test_cannot_confirm_confirmed_receipt() {
        let mut receipt = create_test_receipt();
        let item = create_test_item(receipt.id());
        receipt.add_item(item).unwrap();
        receipt.confirm(UserId::new()).unwrap();

        let result = receipt.confirm(UserId::new());

        assert!(matches!(result, Err(PurchasingError::InvalidStatusTransition)));
    }

    #[test]
    fn test_cannot_cancel_confirmed_receipt() {
        let mut receipt = create_test_receipt();
        let item = create_test_item(receipt.id());
        receipt.add_item(item).unwrap();
        receipt.confirm(UserId::new()).unwrap();

        let result = receipt.cancel();

        assert!(matches!(result, Err(PurchasingError::InvalidStatusTransition)));
    }
}
