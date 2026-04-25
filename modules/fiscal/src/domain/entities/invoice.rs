//! Invoice entity - represents a fiscal invoice document

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::FiscalError;
use crate::domain::entities::InvoiceLine;
use crate::domain::value_objects::{InvoiceId, InvoiceStatus, InvoiceType};
use identity::{StoreId, UserId};
use inventory::Currency;
use pos_core::TerminalId;
use sales::{CustomerId, SaleId};

/// Invoice entity representing a fiscal invoice document.
///
/// Invariants:
/// - Only emitted invoices can be voided
/// - Voiding requires a reason and the user who voided it
/// - Credit notes must reference an original invoice
/// - Invoice numbers are unique within a store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    id: InvoiceId,
    invoice_number: String,
    store_id: StoreId,
    terminal_id: TerminalId,
    sale_id: SaleId,
    cai_range_id: uuid::Uuid,
    invoice_type: InvoiceType,
    status: InvoiceStatus,
    customer_id: Option<CustomerId>,
    customer_name: String,
    customer_rtn: Option<String>,
    customer_address: Option<String>,
    currency: Currency,
    subtotal: Decimal,
    exempt_amount: Decimal,
    taxable_amount_15: Decimal,
    taxable_amount_18: Decimal,
    tax_15: Decimal,
    tax_18: Decimal,
    total_tax: Decimal,
    discount_amount: Decimal,
    total: Decimal,
    amount_in_words: String,
    payment_method: String,
    cai_number: String,
    cai_expiry_date: DateTime<Utc>,
    range_start: String,
    range_end: String,
    voided_by_id: Option<UserId>,
    voided_at: Option<DateTime<Utc>>,
    void_reason: Option<String>,
    void_invoice_id: Option<InvoiceId>,
    original_invoice_id: Option<InvoiceId>,
    printed_at: Option<DateTime<Utc>>,
    emitted_at: DateTime<Utc>,
    items: Vec<InvoiceLine>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Invoice {
    /// Creates a new Invoice
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        invoice_number: String,
        store_id: StoreId,
        terminal_id: TerminalId,
        sale_id: SaleId,
        cai_range_id: uuid::Uuid,
        invoice_type: InvoiceType,
        customer_id: Option<CustomerId>,
        customer_name: String,
        customer_rtn: Option<String>,
        customer_address: Option<String>,
        currency: Currency,
        subtotal: Decimal,
        exempt_amount: Decimal,
        taxable_amount_15: Decimal,
        taxable_amount_18: Decimal,
        tax_15: Decimal,
        tax_18: Decimal,
        total_tax: Decimal,
        discount_amount: Decimal,
        total: Decimal,
        amount_in_words: String,
        payment_method: String,
        cai_number: String,
        cai_expiry_date: DateTime<Utc>,
        range_start: String,
        range_end: String,
        original_invoice_id: Option<InvoiceId>,
        items: Vec<InvoiceLine>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: InvoiceId::new(),
            invoice_number,
            store_id,
            terminal_id,
            sale_id,
            cai_range_id,
            invoice_type,
            status: InvoiceStatus::Emitted,
            customer_id,
            customer_name,
            customer_rtn,
            customer_address,
            currency,
            subtotal,
            exempt_amount,
            taxable_amount_15,
            taxable_amount_18,
            tax_15,
            tax_18,
            total_tax,
            discount_amount,
            total,
            amount_in_words,
            payment_method,
            cai_number,
            cai_expiry_date,
            range_start,
            range_end,
            voided_by_id: None,
            voided_at: None,
            void_reason: None,
            void_invoice_id: None,
            original_invoice_id,
            printed_at: None,
            emitted_at: now,
            items,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes an Invoice from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: InvoiceId,
        invoice_number: String,
        store_id: StoreId,
        terminal_id: TerminalId,
        sale_id: SaleId,
        cai_range_id: uuid::Uuid,
        invoice_type: InvoiceType,
        status: InvoiceStatus,
        customer_id: Option<CustomerId>,
        customer_name: String,
        customer_rtn: Option<String>,
        customer_address: Option<String>,
        currency: Currency,
        subtotal: Decimal,
        exempt_amount: Decimal,
        taxable_amount_15: Decimal,
        taxable_amount_18: Decimal,
        tax_15: Decimal,
        tax_18: Decimal,
        total_tax: Decimal,
        discount_amount: Decimal,
        total: Decimal,
        amount_in_words: String,
        payment_method: String,
        cai_number: String,
        cai_expiry_date: DateTime<Utc>,
        range_start: String,
        range_end: String,
        voided_by_id: Option<UserId>,
        voided_at: Option<DateTime<Utc>>,
        void_reason: Option<String>,
        void_invoice_id: Option<InvoiceId>,
        original_invoice_id: Option<InvoiceId>,
        printed_at: Option<DateTime<Utc>>,
        emitted_at: DateTime<Utc>,
        items: Vec<InvoiceLine>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            invoice_number,
            store_id,
            terminal_id,
            sale_id,
            cai_range_id,
            invoice_type,
            status,
            customer_id,
            customer_name,
            customer_rtn,
            customer_address,
            currency,
            subtotal,
            exempt_amount,
            taxable_amount_15,
            taxable_amount_18,
            tax_15,
            tax_18,
            total_tax,
            discount_amount,
            total,
            amount_in_words,
            payment_method,
            cai_number,
            cai_expiry_date,
            range_start,
            range_end,
            voided_by_id,
            voided_at,
            void_reason,
            void_invoice_id,
            original_invoice_id,
            printed_at,
            emitted_at,
            items,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Workflow Methods
    // =========================================================================

    /// Voids this invoice
    pub fn void(&mut self, voided_by: UserId, reason: String) -> Result<(), FiscalError> {
        if !self.status.can_void() {
            return Err(FiscalError::InvoiceCannotBeVoided);
        }

        let now = Utc::now();
        self.status = InvoiceStatus::Voided;
        self.voided_by_id = Some(voided_by);
        self.voided_at = Some(now);
        self.void_reason = Some(reason);
        self.updated_at = now;
        Ok(())
    }

    /// Marks the invoice as printed
    pub fn mark_printed(&mut self) {
        let now = Utc::now();
        self.printed_at = Some(now);
        self.updated_at = now;
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Returns true if the invoice has been voided
    pub fn is_voided(&self) -> bool {
        self.status == InvoiceStatus::Voided
    }

    /// Returns true if this is a credit note
    pub fn is_credit_note(&self) -> bool {
        self.invoice_type.is_credit_note()
    }

    /// Sets the invoice ID (used when pre-computing ID for line consistency)
    pub fn set_id(&mut self, id: InvoiceId) {
        self.id = id;
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> InvoiceId {
        self.id
    }

    pub fn invoice_number(&self) -> &str {
        &self.invoice_number
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn terminal_id(&self) -> TerminalId {
        self.terminal_id
    }

    pub fn sale_id(&self) -> SaleId {
        self.sale_id
    }

    pub fn cai_range_id(&self) -> uuid::Uuid {
        self.cai_range_id
    }

    pub fn invoice_type(&self) -> InvoiceType {
        self.invoice_type
    }

    pub fn status(&self) -> InvoiceStatus {
        self.status
    }

    pub fn customer_id(&self) -> Option<CustomerId> {
        self.customer_id
    }

    pub fn customer_name(&self) -> &str {
        &self.customer_name
    }

    pub fn customer_rtn(&self) -> Option<&str> {
        self.customer_rtn.as_deref()
    }

    pub fn customer_address(&self) -> Option<&str> {
        self.customer_address.as_deref()
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn subtotal(&self) -> Decimal {
        self.subtotal
    }

    pub fn exempt_amount(&self) -> Decimal {
        self.exempt_amount
    }

    pub fn taxable_amount_15(&self) -> Decimal {
        self.taxable_amount_15
    }

    pub fn taxable_amount_18(&self) -> Decimal {
        self.taxable_amount_18
    }

    pub fn tax_15(&self) -> Decimal {
        self.tax_15
    }

    pub fn tax_18(&self) -> Decimal {
        self.tax_18
    }

    pub fn total_tax(&self) -> Decimal {
        self.total_tax
    }

    pub fn discount_amount(&self) -> Decimal {
        self.discount_amount
    }

    pub fn total(&self) -> Decimal {
        self.total
    }

    pub fn amount_in_words(&self) -> &str {
        &self.amount_in_words
    }

    pub fn payment_method(&self) -> &str {
        &self.payment_method
    }

    pub fn cai_number(&self) -> &str {
        &self.cai_number
    }

    pub fn cai_expiry_date(&self) -> DateTime<Utc> {
        self.cai_expiry_date
    }

    pub fn range_start(&self) -> &str {
        &self.range_start
    }

    pub fn range_end(&self) -> &str {
        &self.range_end
    }

    pub fn voided_by_id(&self) -> Option<UserId> {
        self.voided_by_id
    }

    pub fn voided_at(&self) -> Option<DateTime<Utc>> {
        self.voided_at
    }

    pub fn void_reason(&self) -> Option<&str> {
        self.void_reason.as_deref()
    }

    pub fn void_invoice_id(&self) -> Option<InvoiceId> {
        self.void_invoice_id
    }

    pub fn original_invoice_id(&self) -> Option<InvoiceId> {
        self.original_invoice_id
    }

    pub fn printed_at(&self) -> Option<DateTime<Utc>> {
        self.printed_at
    }

    pub fn emitted_at(&self) -> DateTime<Utc> {
        self.emitted_at
    }

    pub fn items(&self) -> &[InvoiceLine] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<InvoiceLine> {
        &mut self.items
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
