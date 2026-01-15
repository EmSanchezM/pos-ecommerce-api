// StockTransfer entity - document for moving inventory between stores

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::entities::TransferItem;
use crate::domain::value_objects::{TransferId, TransferStatus};
use crate::InventoryError;
use identity::{StoreId, UserId};

/// StockTransfer entity representing a document for transferring inventory between stores.
/// Implements a shipping workflow: draft → pending → in_transit → completed
///
/// Invariants:
/// - from_store_id must not equal to_store_id
/// - Status transitions must follow the defined workflow
/// - Must have items before submitting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTransfer {
    id: TransferId,
    transfer_number: String,
    from_store_id: StoreId,
    to_store_id: StoreId,
    status: TransferStatus,
    requested_date: DateTime<Utc>,
    shipped_date: Option<DateTime<Utc>>,
    received_date: Option<DateTime<Utc>>,
    requested_by_id: UserId,
    shipped_by_id: Option<UserId>,
    received_by_id: Option<UserId>,
    notes: Option<String>,
    shipping_method: Option<String>,
    tracking_number: Option<String>,
    items: Vec<TransferItem>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl StockTransfer {
    /// Creates a new StockTransfer in draft status
    /// Returns error if from_store_id equals to_store_id
    pub fn create(
        transfer_number: String,
        from_store_id: StoreId,
        to_store_id: StoreId,
        requested_by_id: UserId,
    ) -> Result<Self, InventoryError> {
        if from_store_id == to_store_id {
            return Err(InventoryError::SameStoreTransfer);
        }
        
        let now = Utc::now();
        Ok(Self {
            id: TransferId::new(),
            transfer_number,
            from_store_id,
            to_store_id,
            status: TransferStatus::Draft,
            requested_date: now,
            shipped_date: None,
            received_date: None,
            requested_by_id,
            shipped_by_id: None,
            received_by_id: None,
            notes: None,
            shipping_method: None,
            tracking_number: None,
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitutes a StockTransfer from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: TransferId,
        transfer_number: String,
        from_store_id: StoreId,
        to_store_id: StoreId,
        status: TransferStatus,
        requested_date: DateTime<Utc>,
        shipped_date: Option<DateTime<Utc>>,
        received_date: Option<DateTime<Utc>>,
        requested_by_id: UserId,
        shipped_by_id: Option<UserId>,
        received_by_id: Option<UserId>,
        notes: Option<String>,
        shipping_method: Option<String>,
        tracking_number: Option<String>,
        items: Vec<TransferItem>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            transfer_number,
            from_store_id,
            to_store_id,
            status,
            requested_date,
            shipped_date,
            received_date,
            requested_by_id,
            shipped_by_id,
            received_by_id,
            notes,
            shipping_method,
            tracking_number,
            items,
            created_at,
            updated_at,
        }
    }

    /// Submits the transfer for processing
    /// Transitions: draft → pending
    pub fn submit(&mut self) -> Result<(), InventoryError> {
        if self.status != TransferStatus::Draft {
            return Err(InventoryError::InvalidStatusTransition);
        }
        if self.items.is_empty() {
            return Err(InventoryError::EmptyTransfer);
        }
        self.status = TransferStatus::Pending;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Ships the transfer
    /// Transitions: pending → in_transit
    pub fn ship(&mut self, shipped_by_id: UserId) -> Result<(), InventoryError> {
        if self.status != TransferStatus::Pending {
            return Err(InventoryError::InvalidStatusTransition);
        }
        self.status = TransferStatus::InTransit;
        self.shipped_by_id = Some(shipped_by_id);
        self.shipped_date = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Receives the transfer at destination
    /// Transitions: in_transit → completed
    pub fn receive(&mut self, received_by_id: UserId) -> Result<(), InventoryError> {
        if self.status != TransferStatus::InTransit {
            return Err(InventoryError::InvalidStatusTransition);
        }
        self.status = TransferStatus::Completed;
        self.received_by_id = Some(received_by_id);
        self.received_date = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Completes the transfer (alias for receive for clarity)
    pub fn complete(&mut self, received_by_id: UserId) -> Result<(), InventoryError> {
        self.receive(received_by_id)
    }

    /// Cancels the transfer
    /// Transitions: draft/pending → cancelled
    pub fn cancel(&mut self) -> Result<(), InventoryError> {
        if !matches!(self.status, TransferStatus::Draft | TransferStatus::Pending) {
            return Err(InventoryError::InvalidStatusTransition);
        }
        self.status = TransferStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Adds an item to the transfer (only allowed in draft status)
    pub fn add_item(&mut self, item: TransferItem) -> Result<(), InventoryError> {
        if self.status != TransferStatus::Draft {
            return Err(InventoryError::InvalidStatusTransition);
        }
        self.items.push(item);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Returns true if the transfer can be modified
    pub fn is_editable(&self) -> bool {
        self.status == TransferStatus::Draft
    }

    /// Returns true if the transfer is in a final state
    pub fn is_final(&self) -> bool {
        matches!(
            self.status,
            TransferStatus::Completed | TransferStatus::Cancelled
        )
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> TransferId {
        self.id
    }

    pub fn transfer_number(&self) -> &str {
        &self.transfer_number
    }

    pub fn from_store_id(&self) -> StoreId {
        self.from_store_id
    }

    pub fn to_store_id(&self) -> StoreId {
        self.to_store_id
    }

    pub fn status(&self) -> TransferStatus {
        self.status
    }

    pub fn requested_date(&self) -> DateTime<Utc> {
        self.requested_date
    }

    pub fn shipped_date(&self) -> Option<DateTime<Utc>> {
        self.shipped_date
    }

    pub fn received_date(&self) -> Option<DateTime<Utc>> {
        self.received_date
    }

    pub fn requested_by_id(&self) -> UserId {
        self.requested_by_id
    }

    pub fn shipped_by_id(&self) -> Option<UserId> {
        self.shipped_by_id
    }

    pub fn received_by_id(&self) -> Option<UserId> {
        self.received_by_id
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn shipping_method(&self) -> Option<&str> {
        self.shipping_method.as_deref()
    }

    pub fn tracking_number(&self) -> Option<&str> {
        self.tracking_number.as_deref()
    }

    pub fn items(&self) -> &[TransferItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<TransferItem> {
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

    pub fn set_notes(&mut self, notes: Option<String>) -> Result<(), InventoryError> {
        if !self.is_editable() {
            return Err(InventoryError::InvalidStatusTransition);
        }
        self.notes = notes;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_shipping_method(&mut self, method: Option<String>) -> Result<(), InventoryError> {
        if !self.is_editable() {
            return Err(InventoryError::InvalidStatusTransition);
        }
        self.shipping_method = method;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_tracking_number(&mut self, tracking: Option<String>) -> Result<(), InventoryError> {
        // Tracking number can be set until shipped
        if self.is_final() {
            return Err(InventoryError::InvalidStatusTransition);
        }
        self.tracking_number = tracking;
        self.updated_at = Utc::now();
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::domain::value_objects::ProductId;

    fn create_test_transfer() -> StockTransfer {
        let from_store = StoreId::new();
        let to_store = StoreId::new();
        StockTransfer::create(
            "TRF-001".to_string(),
            from_store,
            to_store,
            UserId::new(),
        ).unwrap()
    }

    fn create_test_item() -> TransferItem {
        TransferItem::create_for_product(
            TransferId::new(),
            ProductId::new(),
            dec!(10),
            Some(dec!(5.00)),
        ).unwrap()
    }

    #[test]
    fn test_create_transfer() {
        let from_store = StoreId::new();
        let to_store = StoreId::new();
        let user_id = UserId::new();
        
        let transfer = StockTransfer::create(
            "TRF-001".to_string(),
            from_store,
            to_store,
            user_id,
        ).unwrap();
        
        assert_eq!(transfer.transfer_number(), "TRF-001");
        assert_eq!(transfer.from_store_id(), from_store);
        assert_eq!(transfer.to_store_id(), to_store);
        assert_eq!(transfer.status(), TransferStatus::Draft);
        assert_eq!(transfer.requested_by_id(), user_id);
        assert!(transfer.shipped_by_id().is_none());
        assert!(transfer.received_by_id().is_none());
        assert!(transfer.shipped_date().is_none());
        assert!(transfer.received_date().is_none());
        assert!(transfer.items().is_empty());
        assert!(transfer.is_editable());
        assert!(!transfer.is_final());
    }

    #[test]
    fn test_create_transfer_same_store() {
        let store_id = StoreId::new();
        
        let result = StockTransfer::create(
            "TRF-001".to_string(),
            store_id,
            store_id,
            UserId::new(),
        );
        
        assert!(matches!(result, Err(InventoryError::SameStoreTransfer)));
    }

    #[test]
    fn test_add_item_in_draft() {
        let mut transfer = create_test_transfer();
        let item = create_test_item();
        
        transfer.add_item(item).unwrap();
        
        assert_eq!(transfer.items().len(), 1);
    }

    #[test]
    fn test_submit_success() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        
        transfer.submit().unwrap();
        
        assert_eq!(transfer.status(), TransferStatus::Pending);
        assert!(!transfer.is_editable());
    }

    #[test]
    fn test_submit_empty_items() {
        let mut transfer = create_test_transfer();
        
        let result = transfer.submit();
        
        assert!(matches!(result, Err(InventoryError::EmptyTransfer)));
    }

    #[test]
    fn test_submit_wrong_status() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        
        let result = transfer.submit();
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_ship_success() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        
        let shipper_id = UserId::new();
        transfer.ship(shipper_id).unwrap();
        
        assert_eq!(transfer.status(), TransferStatus::InTransit);
        assert_eq!(transfer.shipped_by_id(), Some(shipper_id));
        assert!(transfer.shipped_date().is_some());
    }

    #[test]
    fn test_ship_wrong_status() {
        let mut transfer = create_test_transfer();
        
        let result = transfer.ship(UserId::new());
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_receive_success() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        transfer.ship(UserId::new()).unwrap();
        
        let receiver_id = UserId::new();
        transfer.receive(receiver_id).unwrap();
        
        assert_eq!(transfer.status(), TransferStatus::Completed);
        assert_eq!(transfer.received_by_id(), Some(receiver_id));
        assert!(transfer.received_date().is_some());
        assert!(transfer.is_final());
    }

    #[test]
    fn test_receive_wrong_status() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        
        let result = transfer.receive(UserId::new());
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_cancel_from_draft() {
        let mut transfer = create_test_transfer();
        
        transfer.cancel().unwrap();
        
        assert_eq!(transfer.status(), TransferStatus::Cancelled);
        assert!(transfer.is_final());
    }

    #[test]
    fn test_cancel_from_pending() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        
        transfer.cancel().unwrap();
        
        assert_eq!(transfer.status(), TransferStatus::Cancelled);
    }

    #[test]
    fn test_cancel_from_in_transit() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        transfer.ship(UserId::new()).unwrap();
        
        let result = transfer.cancel();
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_add_item_after_submit() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        
        let result = transfer.add_item(create_test_item());
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_set_notes_in_draft() {
        let mut transfer = create_test_transfer();
        
        transfer.set_notes(Some("Test notes".to_string())).unwrap();
        
        assert_eq!(transfer.notes(), Some("Test notes"));
    }

    #[test]
    fn test_set_notes_after_submit() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        
        let result = transfer.set_notes(Some("Test notes".to_string()));
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_set_tracking_number_before_complete() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        transfer.ship(UserId::new()).unwrap();
        
        transfer.set_tracking_number(Some("TRACK123".to_string())).unwrap();
        
        assert_eq!(transfer.tracking_number(), Some("TRACK123"));
    }

    #[test]
    fn test_set_tracking_number_after_complete() {
        let mut transfer = create_test_transfer();
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        transfer.ship(UserId::new()).unwrap();
        transfer.receive(UserId::new()).unwrap();
        
        let result = transfer.set_tracking_number(Some("TRACK123".to_string()));
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_workflow_draft_to_completed() {
        let mut transfer = create_test_transfer();
        
        // Draft
        assert_eq!(transfer.status(), TransferStatus::Draft);
        assert!(transfer.is_editable());
        
        // Add item and submit
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        assert_eq!(transfer.status(), TransferStatus::Pending);
        
        // Ship
        transfer.ship(UserId::new()).unwrap();
        assert_eq!(transfer.status(), TransferStatus::InTransit);
        
        // Receive/Complete
        transfer.receive(UserId::new()).unwrap();
        assert_eq!(transfer.status(), TransferStatus::Completed);
        assert!(transfer.is_final());
    }

    #[test]
    fn test_workflow_draft_to_cancelled() {
        let mut transfer = create_test_transfer();
        
        transfer.add_item(create_test_item()).unwrap();
        transfer.submit().unwrap();
        transfer.cancel().unwrap();
        
        assert_eq!(transfer.status(), TransferStatus::Cancelled);
        assert!(transfer.is_final());
    }
}
