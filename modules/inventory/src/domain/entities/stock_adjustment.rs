// StockAdjustment entity - document for correcting inventory with approval workflow

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::entities::AdjustmentItem;
use crate::domain::value_objects::{
    AdjustmentId, AdjustmentReason, AdjustmentStatus, AdjustmentType,
};
use crate::InventoryError;
use identity::{StoreId, UserId};

/// StockAdjustment entity representing a document for correcting inventory.
/// Implements an approval workflow: draft → pending_approval → approved/rejected → applied
///
/// Invariants:
/// - Status transitions must follow the defined workflow
/// - Applied adjustments cannot be modified
/// - Must have items before submitting for approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAdjustment {
    id: AdjustmentId,
    store_id: StoreId,
    adjustment_number: String,
    adjustment_type: AdjustmentType,
    adjustment_reason: AdjustmentReason,
    status: AdjustmentStatus,
    created_by_id: UserId,
    approved_by_id: Option<UserId>,
    approved_at: Option<DateTime<Utc>>,
    applied_at: Option<DateTime<Utc>>,
    notes: Option<String>,
    attachments: JsonValue,
    items: Vec<AdjustmentItem>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl StockAdjustment {
    /// Creates a new StockAdjustment in draft status
    pub fn create(
        store_id: StoreId,
        adjustment_number: String,
        adjustment_type: AdjustmentType,
        adjustment_reason: AdjustmentReason,
        created_by_id: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: AdjustmentId::new(),
            store_id,
            adjustment_number,
            adjustment_type,
            adjustment_reason,
            status: AdjustmentStatus::Draft,
            created_by_id,
            approved_by_id: None,
            approved_at: None,
            applied_at: None,
            notes: None,
            attachments: JsonValue::Array(vec![]),
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a StockAdjustment from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: AdjustmentId,
        store_id: StoreId,
        adjustment_number: String,
        adjustment_type: AdjustmentType,
        adjustment_reason: AdjustmentReason,
        status: AdjustmentStatus,
        created_by_id: UserId,
        approved_by_id: Option<UserId>,
        approved_at: Option<DateTime<Utc>>,
        applied_at: Option<DateTime<Utc>>,
        notes: Option<String>,
        attachments: JsonValue,
        items: Vec<AdjustmentItem>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            adjustment_number,
            adjustment_type,
            adjustment_reason,
            status,
            created_by_id,
            approved_by_id,
            approved_at,
            applied_at,
            notes,
            attachments,
            items,
            created_at,
            updated_at,
        }
    }

    /// Submits the adjustment for approval
    /// Transitions: draft → pending_approval
    pub fn submit_for_approval(&mut self) -> Result<(), InventoryError> {
        if self.status != AdjustmentStatus::Draft {
            return Err(InventoryError::InvalidStatusTransition);
        }
        if self.items.is_empty() {
            return Err(InventoryError::EmptyAdjustment);
        }
        self.status = AdjustmentStatus::PendingApproval;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Approves the adjustment
    /// Transitions: pending_approval → approved
    pub fn approve(&mut self, approver_id: UserId) -> Result<(), InventoryError> {
        if self.status != AdjustmentStatus::PendingApproval {
            return Err(InventoryError::InvalidStatusTransition);
        }
        self.status = AdjustmentStatus::Approved;
        self.approved_by_id = Some(approver_id);
        self.approved_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Rejects the adjustment
    /// Transitions: pending_approval → rejected
    pub fn reject(&mut self, approver_id: UserId) -> Result<(), InventoryError> {
        if self.status != AdjustmentStatus::PendingApproval {
            return Err(InventoryError::InvalidStatusTransition);
        }
        self.status = AdjustmentStatus::Rejected;
        self.approved_by_id = Some(approver_id);
        self.approved_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Marks the adjustment as applied to inventory
    /// Transitions: approved → applied
    pub fn mark_applied(&mut self) -> Result<(), InventoryError> {
        if self.status != AdjustmentStatus::Approved {
            return Err(InventoryError::InvalidStatusTransition);
        }
        self.status = AdjustmentStatus::Applied;
        self.applied_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Adds an item to the adjustment (only allowed in draft status)
    pub fn add_item(&mut self, item: AdjustmentItem) -> Result<(), InventoryError> {
        if self.status != AdjustmentStatus::Draft {
            return Err(InventoryError::AdjustmentAlreadyApplied);
        }
        self.items.push(item);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Returns true if the adjustment can be modified
    pub fn is_editable(&self) -> bool {
        self.status == AdjustmentStatus::Draft
    }

    /// Returns true if the adjustment is in a final state
    pub fn is_final(&self) -> bool {
        matches!(
            self.status,
            AdjustmentStatus::Rejected | AdjustmentStatus::Applied
        )
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> AdjustmentId {
        self.id
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn adjustment_number(&self) -> &str {
        &self.adjustment_number
    }

    pub fn adjustment_type(&self) -> AdjustmentType {
        self.adjustment_type
    }

    pub fn adjustment_reason(&self) -> AdjustmentReason {
        self.adjustment_reason
    }

    pub fn status(&self) -> AdjustmentStatus {
        self.status
    }

    pub fn created_by_id(&self) -> UserId {
        self.created_by_id
    }

    pub fn approved_by_id(&self) -> Option<UserId> {
        self.approved_by_id
    }

    pub fn approved_at(&self) -> Option<DateTime<Utc>> {
        self.approved_at
    }

    pub fn applied_at(&self) -> Option<DateTime<Utc>> {
        self.applied_at
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn attachments(&self) -> &JsonValue {
        &self.attachments
    }

    pub fn items(&self) -> &[AdjustmentItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<AdjustmentItem> {
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
            return Err(InventoryError::AdjustmentAlreadyApplied);
        }
        self.notes = notes;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_attachments(&mut self, attachments: JsonValue) -> Result<(), InventoryError> {
        if !self.is_editable() {
            return Err(InventoryError::AdjustmentAlreadyApplied);
        }
        self.attachments = attachments;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_adjustment_type(&mut self, adjustment_type: AdjustmentType) -> Result<(), InventoryError> {
        if !self.is_editable() {
            return Err(InventoryError::AdjustmentAlreadyApplied);
        }
        self.adjustment_type = adjustment_type;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_adjustment_reason(&mut self, adjustment_reason: AdjustmentReason) -> Result<(), InventoryError> {
        if !self.is_editable() {
            return Err(InventoryError::AdjustmentAlreadyApplied);
        }
        self.adjustment_reason = adjustment_reason;
        self.updated_at = Utc::now();
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::domain::value_objects::StockId;

    fn create_test_adjustment() -> StockAdjustment {
        StockAdjustment::create(
            StoreId::new(),
            "ADJ-001".to_string(),
            AdjustmentType::Decrease,
            AdjustmentReason::Damage,
            UserId::new(),
        )
    }

    fn create_test_item() -> AdjustmentItem {
        AdjustmentItem::create(
            AdjustmentId::new(),
            StockId::new(),
            dec!(-5),
            Some(dec!(10.00)),
        )
    }

    #[test]
    fn test_create_adjustment() {
        let store_id = StoreId::new();
        let user_id = UserId::new();
        
        let adjustment = StockAdjustment::create(
            store_id,
            "ADJ-001".to_string(),
            AdjustmentType::Decrease,
            AdjustmentReason::Damage,
            user_id,
        );
        
        assert_eq!(adjustment.store_id(), store_id);
        assert_eq!(adjustment.adjustment_number(), "ADJ-001");
        assert_eq!(adjustment.adjustment_type(), AdjustmentType::Decrease);
        assert_eq!(adjustment.adjustment_reason(), AdjustmentReason::Damage);
        assert_eq!(adjustment.status(), AdjustmentStatus::Draft);
        assert_eq!(adjustment.created_by_id(), user_id);
        assert!(adjustment.approved_by_id().is_none());
        assert!(adjustment.approved_at().is_none());
        assert!(adjustment.applied_at().is_none());
        assert!(adjustment.items().is_empty());
        assert!(adjustment.is_editable());
        assert!(!adjustment.is_final());
    }

    #[test]
    fn test_add_item_in_draft() {
        let mut adjustment = create_test_adjustment();
        let item = create_test_item();
        
        adjustment.add_item(item).unwrap();
        
        assert_eq!(adjustment.items().len(), 1);
    }

    #[test]
    fn test_submit_for_approval_success() {
        let mut adjustment = create_test_adjustment();
        adjustment.add_item(create_test_item()).unwrap();
        
        adjustment.submit_for_approval().unwrap();
        
        assert_eq!(adjustment.status(), AdjustmentStatus::PendingApproval);
        assert!(!adjustment.is_editable());
    }

    #[test]
    fn test_submit_for_approval_empty_items() {
        let mut adjustment = create_test_adjustment();
        
        let result = adjustment.submit_for_approval();
        
        assert!(matches!(result, Err(InventoryError::EmptyAdjustment)));
    }

    #[test]
    fn test_submit_for_approval_wrong_status() {
        let mut adjustment = create_test_adjustment();
        adjustment.add_item(create_test_item()).unwrap();
        adjustment.submit_for_approval().unwrap();
        
        // Try to submit again
        let result = adjustment.submit_for_approval();
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_approve_success() {
        let mut adjustment = create_test_adjustment();
        adjustment.add_item(create_test_item()).unwrap();
        adjustment.submit_for_approval().unwrap();
        
        let approver_id = UserId::new();
        adjustment.approve(approver_id).unwrap();
        
        assert_eq!(adjustment.status(), AdjustmentStatus::Approved);
        assert_eq!(adjustment.approved_by_id(), Some(approver_id));
        assert!(adjustment.approved_at().is_some());
    }

    #[test]
    fn test_approve_wrong_status() {
        let mut adjustment = create_test_adjustment();
        
        let result = adjustment.approve(UserId::new());
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_reject_success() {
        let mut adjustment = create_test_adjustment();
        adjustment.add_item(create_test_item()).unwrap();
        adjustment.submit_for_approval().unwrap();
        
        let approver_id = UserId::new();
        adjustment.reject(approver_id).unwrap();
        
        assert_eq!(adjustment.status(), AdjustmentStatus::Rejected);
        assert_eq!(adjustment.approved_by_id(), Some(approver_id));
        assert!(adjustment.approved_at().is_some());
        assert!(adjustment.is_final());
    }

    #[test]
    fn test_reject_wrong_status() {
        let mut adjustment = create_test_adjustment();
        
        let result = adjustment.reject(UserId::new());
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_mark_applied_success() {
        let mut adjustment = create_test_adjustment();
        adjustment.add_item(create_test_item()).unwrap();
        adjustment.submit_for_approval().unwrap();
        adjustment.approve(UserId::new()).unwrap();
        
        adjustment.mark_applied().unwrap();
        
        assert_eq!(adjustment.status(), AdjustmentStatus::Applied);
        assert!(adjustment.applied_at().is_some());
        assert!(adjustment.is_final());
    }

    #[test]
    fn test_mark_applied_wrong_status() {
        let mut adjustment = create_test_adjustment();
        adjustment.add_item(create_test_item()).unwrap();
        adjustment.submit_for_approval().unwrap();
        
        // Try to apply without approval
        let result = adjustment.mark_applied();
        
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }

    #[test]
    fn test_add_item_after_submit() {
        let mut adjustment = create_test_adjustment();
        adjustment.add_item(create_test_item()).unwrap();
        adjustment.submit_for_approval().unwrap();
        
        let result = adjustment.add_item(create_test_item());
        
        assert!(matches!(result, Err(InventoryError::AdjustmentAlreadyApplied)));
    }

    #[test]
    fn test_set_notes_in_draft() {
        let mut adjustment = create_test_adjustment();
        
        adjustment.set_notes(Some("Test notes".to_string())).unwrap();
        
        assert_eq!(adjustment.notes(), Some("Test notes"));
    }

    #[test]
    fn test_set_notes_after_submit() {
        let mut adjustment = create_test_adjustment();
        adjustment.add_item(create_test_item()).unwrap();
        adjustment.submit_for_approval().unwrap();
        
        let result = adjustment.set_notes(Some("Test notes".to_string()));
        
        assert!(matches!(result, Err(InventoryError::AdjustmentAlreadyApplied)));
    }

    #[test]
    fn test_workflow_draft_to_applied() {
        let mut adjustment = create_test_adjustment();
        
        // Draft
        assert_eq!(adjustment.status(), AdjustmentStatus::Draft);
        assert!(adjustment.is_editable());
        
        // Add item and submit
        adjustment.add_item(create_test_item()).unwrap();
        adjustment.submit_for_approval().unwrap();
        assert_eq!(adjustment.status(), AdjustmentStatus::PendingApproval);
        
        // Approve
        adjustment.approve(UserId::new()).unwrap();
        assert_eq!(adjustment.status(), AdjustmentStatus::Approved);
        
        // Apply
        adjustment.mark_applied().unwrap();
        assert_eq!(adjustment.status(), AdjustmentStatus::Applied);
        assert!(adjustment.is_final());
    }

    #[test]
    fn test_workflow_draft_to_rejected() {
        let mut adjustment = create_test_adjustment();
        
        adjustment.add_item(create_test_item()).unwrap();
        adjustment.submit_for_approval().unwrap();
        adjustment.reject(UserId::new()).unwrap();
        
        assert_eq!(adjustment.status(), AdjustmentStatus::Rejected);
        assert!(adjustment.is_final());
    }
}
