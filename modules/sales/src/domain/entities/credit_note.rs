//! CreditNote entity - represents a return/refund document

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::entities::CreditNoteItem;
use crate::domain::value_objects::{
    CreditNoteId, CreditNoteItemId, CreditNoteStatus, ReturnReason, ReturnType, SaleId,
};
use crate::SalesError;
use identity::{StoreId, UserId};
use inventory::Currency;

/// CreditNote entity representing a return/refund document.
///
/// Invariants:
/// - Must reference an original completed sale
/// - Return quantities cannot exceed original sale quantities
/// - User cannot approve their own credit note
/// - Must have items before submitting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditNote {
    id: CreditNoteId,
    credit_note_number: String,
    store_id: StoreId,
    original_sale_id: SaleId,
    original_invoice_number: String,
    status: CreditNoteStatus,
    return_type: ReturnType,
    return_reason: ReturnReason,
    reason_details: Option<String>,
    currency: Currency,
    subtotal: Decimal,
    tax_amount: Decimal,
    total: Decimal,
    refund_method: Option<String>,
    refunded_amount: Decimal,
    created_by_id: UserId,
    submitted_by_id: Option<UserId>,
    submitted_at: Option<DateTime<Utc>>,
    approved_by_id: Option<UserId>,
    approved_at: Option<DateTime<Utc>>,
    applied_by_id: Option<UserId>,
    applied_at: Option<DateTime<Utc>>,
    cancelled_by_id: Option<UserId>,
    cancelled_at: Option<DateTime<Utc>>,
    cancellation_reason: Option<String>,
    notes: Option<String>,
    items: Vec<CreditNoteItem>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CreditNote {
    /// Creates a new CreditNote
    pub fn create(
        credit_note_number: String,
        store_id: StoreId,
        original_sale_id: SaleId,
        original_invoice_number: String,
        return_type: ReturnType,
        return_reason: ReturnReason,
        currency: Currency,
        created_by_id: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: CreditNoteId::new(),
            credit_note_number,
            store_id,
            original_sale_id,
            original_invoice_number,
            status: CreditNoteStatus::Draft,
            return_type,
            return_reason,
            reason_details: None,
            currency,
            subtotal: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            total: Decimal::ZERO,
            refund_method: None,
            refunded_amount: Decimal::ZERO,
            created_by_id,
            submitted_by_id: None,
            submitted_at: None,
            approved_by_id: None,
            approved_at: None,
            applied_by_id: None,
            applied_at: None,
            cancelled_by_id: None,
            cancelled_at: None,
            cancellation_reason: None,
            notes: None,
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a CreditNote from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: CreditNoteId,
        credit_note_number: String,
        store_id: StoreId,
        original_sale_id: SaleId,
        original_invoice_number: String,
        status: CreditNoteStatus,
        return_type: ReturnType,
        return_reason: ReturnReason,
        reason_details: Option<String>,
        currency: Currency,
        subtotal: Decimal,
        tax_amount: Decimal,
        total: Decimal,
        refund_method: Option<String>,
        refunded_amount: Decimal,
        created_by_id: UserId,
        submitted_by_id: Option<UserId>,
        submitted_at: Option<DateTime<Utc>>,
        approved_by_id: Option<UserId>,
        approved_at: Option<DateTime<Utc>>,
        applied_by_id: Option<UserId>,
        applied_at: Option<DateTime<Utc>>,
        cancelled_by_id: Option<UserId>,
        cancelled_at: Option<DateTime<Utc>>,
        cancellation_reason: Option<String>,
        notes: Option<String>,
        items: Vec<CreditNoteItem>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            credit_note_number,
            store_id,
            original_sale_id,
            original_invoice_number,
            status,
            return_type,
            return_reason,
            reason_details,
            currency,
            subtotal,
            tax_amount,
            total,
            refund_method,
            refunded_amount,
            created_by_id,
            submitted_by_id,
            submitted_at,
            approved_by_id,
            approved_at,
            applied_by_id,
            applied_at,
            cancelled_by_id,
            cancelled_at,
            cancellation_reason,
            notes,
            items,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Workflow Methods
    // =========================================================================

    /// Submits the credit note for approval
    pub fn submit(&mut self, submitted_by_id: UserId) -> Result<(), SalesError> {
        if !self.status.can_submit() {
            return Err(SalesError::InvalidStatusTransition);
        }
        if self.items.is_empty() {
            return Err(SalesError::EmptyCreditNote);
        }

        self.status = CreditNoteStatus::Pending;
        self.submitted_by_id = Some(submitted_by_id);
        self.submitted_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Approves the credit note
    pub fn approve(&mut self, approver_id: UserId) -> Result<(), SalesError> {
        if !self.status.can_approve() {
            return Err(SalesError::InvalidStatusTransition);
        }
        // User cannot approve their own credit note
        if approver_id == self.created_by_id {
            return Err(SalesError::CannotApproveSelfCreatedCreditNote);
        }

        self.status = CreditNoteStatus::Approved;
        self.approved_by_id = Some(approver_id);
        self.approved_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Applies the credit note (processes refund)
    pub fn apply(
        &mut self,
        applied_by_id: UserId,
        refund_method: String,
    ) -> Result<(), SalesError> {
        if !self.status.can_apply() {
            return Err(SalesError::InvalidStatusTransition);
        }

        self.status = CreditNoteStatus::Applied;
        self.applied_by_id = Some(applied_by_id);
        self.applied_at = Some(Utc::now());
        self.refund_method = Some(refund_method);
        self.refunded_amount = self.total;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancels the credit note
    pub fn cancel(
        &mut self,
        cancelled_by_id: UserId,
        reason: String,
    ) -> Result<(), SalesError> {
        if !self.status.can_cancel() {
            return Err(SalesError::InvalidStatusTransition);
        }

        self.status = CreditNoteStatus::Cancelled;
        self.cancelled_by_id = Some(cancelled_by_id);
        self.cancelled_at = Some(Utc::now());
        self.cancellation_reason = Some(reason);
        self.updated_at = Utc::now();
        Ok(())
    }

    // =========================================================================
    // Item Management
    // =========================================================================

    /// Adds an item to the credit note
    pub fn add_item(&mut self, item: CreditNoteItem) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::CreditNoteNotEditable);
        }
        self.items.push(item);
        self.recalculate_totals();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Removes an item from the credit note
    pub fn remove_item(&mut self, item_id: CreditNoteItemId) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::CreditNoteNotEditable);
        }
        self.items.retain(|i| i.id() != item_id);
        self.recalculate_totals();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Recalculates the credit note totals
    pub fn recalculate_totals(&mut self) {
        self.subtotal = self.items.iter().map(|i| i.subtotal()).sum();
        self.tax_amount = self.items.iter().map(|i| i.tax_amount()).sum();
        self.total = self.subtotal + self.tax_amount;
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Returns true if the credit note can be edited
    pub fn is_editable(&self) -> bool {
        self.status.is_editable()
    }

    /// Returns true if the credit note is in a final state
    pub fn is_final(&self) -> bool {
        self.status.is_final()
    }

    /// Returns the number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> CreditNoteId {
        self.id
    }

    pub fn credit_note_number(&self) -> &str {
        &self.credit_note_number
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn original_sale_id(&self) -> SaleId {
        self.original_sale_id
    }

    pub fn original_invoice_number(&self) -> &str {
        &self.original_invoice_number
    }

    pub fn status(&self) -> CreditNoteStatus {
        self.status
    }

    pub fn return_type(&self) -> ReturnType {
        self.return_type
    }

    pub fn return_reason(&self) -> ReturnReason {
        self.return_reason
    }

    pub fn reason_details(&self) -> Option<&str> {
        self.reason_details.as_deref()
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn subtotal(&self) -> Decimal {
        self.subtotal
    }

    pub fn tax_amount(&self) -> Decimal {
        self.tax_amount
    }

    pub fn total(&self) -> Decimal {
        self.total
    }

    pub fn refund_method(&self) -> Option<&str> {
        self.refund_method.as_deref()
    }

    pub fn refunded_amount(&self) -> Decimal {
        self.refunded_amount
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

    pub fn applied_by_id(&self) -> Option<UserId> {
        self.applied_by_id
    }

    pub fn applied_at(&self) -> Option<DateTime<Utc>> {
        self.applied_at
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

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn items(&self) -> &[CreditNoteItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<CreditNoteItem> {
        &mut self.items
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // =========================================================================
    // Setters
    // =========================================================================

    pub fn set_reason_details(&mut self, details: Option<String>) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::CreditNoteNotEditable);
        }
        self.reason_details = details;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_notes(&mut self, notes: Option<String>) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::CreditNoteNotEditable);
        }
        self.notes = notes;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_credit_note() -> CreditNote {
        CreditNote::create(
            "CN-001".to_string(),
            StoreId::new(),
            SaleId::new(),
            "INV-001".to_string(),
            ReturnType::Partial,
            ReturnReason::Defective,
            Currency::new("USD").unwrap(),
            UserId::new(),
        )
    }

    fn create_test_item(credit_note_id: CreditNoteId) -> CreditNoteItem {
        use inventory::{ProductId, UnitOfMeasure};
        use rust_decimal_macros::dec;
        use std::str::FromStr;

        CreditNoteItem::create(
            credit_note_id,
            SaleItemId::new(),
            ProductId::new(),
            None,
            "SKU-001".to_string(),
            "Test Product".to_string(),
            dec!(1),
            UnitOfMeasure::from_str("unit").unwrap(),
            dec!(50.00),
            dec!(15),
        )
        .unwrap()
    }

    use crate::domain::value_objects::SaleItemId;

    #[test]
    fn test_create_credit_note() {
        let cn = create_test_credit_note();

        assert_eq!(cn.status(), CreditNoteStatus::Draft);
        assert!(cn.is_editable());
        assert!(!cn.is_final());
    }

    #[test]
    fn test_submit_workflow() {
        let mut cn = create_test_credit_note();
        let item = create_test_item(cn.id());
        cn.add_item(item).unwrap();

        cn.submit(UserId::new()).unwrap();

        assert_eq!(cn.status(), CreditNoteStatus::Pending);
        assert!(!cn.is_editable());
    }

    #[test]
    fn test_cannot_submit_empty() {
        let mut cn = create_test_credit_note();

        let result = cn.submit(UserId::new());

        assert!(matches!(result, Err(SalesError::EmptyCreditNote)));
    }

    #[test]
    fn test_approve_workflow() {
        let mut cn = create_test_credit_note();
        let item = create_test_item(cn.id());
        cn.add_item(item).unwrap();
        cn.submit(UserId::new()).unwrap();

        let approver = UserId::new();
        cn.approve(approver).unwrap();

        assert_eq!(cn.status(), CreditNoteStatus::Approved);
        assert_eq!(cn.approved_by_id(), Some(approver));
    }

    #[test]
    fn test_cannot_approve_own_credit_note() {
        let creator = UserId::new();
        let mut cn = CreditNote::create(
            "CN-001".to_string(),
            StoreId::new(),
            SaleId::new(),
            "INV-001".to_string(),
            ReturnType::Partial,
            ReturnReason::Defective,
            Currency::new("USD").unwrap(),
            creator,
        );
        let item = create_test_item(cn.id());
        cn.add_item(item).unwrap();
        cn.submit(creator).unwrap();

        let result = cn.approve(creator);

        assert!(matches!(
            result,
            Err(SalesError::CannotApproveSelfCreatedCreditNote)
        ));
    }

    #[test]
    fn test_apply_workflow() {
        let mut cn = create_test_credit_note();
        let item = create_test_item(cn.id());
        cn.add_item(item).unwrap();
        cn.submit(UserId::new()).unwrap();
        cn.approve(UserId::new()).unwrap();

        cn.apply(UserId::new(), "Cash".to_string()).unwrap();

        assert_eq!(cn.status(), CreditNoteStatus::Applied);
        assert!(cn.is_final());
    }

    #[test]
    fn test_cancel_workflow() {
        let mut cn = create_test_credit_note();

        cn.cancel(UserId::new(), "No longer needed".to_string())
            .unwrap();

        assert_eq!(cn.status(), CreditNoteStatus::Cancelled);
        assert!(cn.is_final());
    }
}
