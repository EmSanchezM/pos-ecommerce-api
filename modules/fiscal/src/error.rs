//! Fiscal module error types.
//!
//! This module defines all error types that can occur during fiscal operations.
//! Errors are categorized by domain area (invoices, tax rates, fiscal sequences).

use thiserror::Error;
use uuid::Uuid;

/// Error type for all fiscal module operations.
///
/// This enum covers all possible error conditions that can occur when working
/// with the fiscal module, including validation errors, not-found errors,
/// and workflow constraint violations.
#[derive(Debug, Error)]
pub enum FiscalError {
    // -------------------------------------------------------------------------
    // Invoice errors
    // -------------------------------------------------------------------------
    /// The requested invoice was not found in the database.
    #[error("Invoice not found: {0}")]
    InvoiceNotFound(Uuid),

    /// An invoice with the given number already exists.
    #[error("Invoice number '{0}' already exists")]
    DuplicateInvoiceNumber(String),

    /// The invoice has already been voided.
    #[error("Invoice has already been voided")]
    InvoiceAlreadyVoided,

    /// The void window for this invoice has expired.
    #[error("Void window for this invoice has expired")]
    VoidWindowExpired,

    /// The invoice cannot be voided in its current status.
    #[error("Invoice cannot be voided")]
    InvoiceCannotBeVoided,

    // -------------------------------------------------------------------------
    // Tax Rate errors
    // -------------------------------------------------------------------------
    /// The requested tax rate was not found in the database.
    #[error("Tax rate not found: {0}")]
    TaxRateNotFound(Uuid),

    /// A tax rate with the given name already exists.
    #[error("Tax rate name '{0}' already exists")]
    DuplicateTaxRateName(String),

    /// Cannot delete a default tax rate.
    #[error("Cannot delete a default tax rate")]
    CannotDeleteDefaultTaxRate,

    // -------------------------------------------------------------------------
    // Fiscal Sequence errors
    // -------------------------------------------------------------------------
    /// No fiscal sequence found for the given configuration.
    #[error("Fiscal sequence not found")]
    FiscalSequenceNotFound,

    /// The fiscal sequence has been exhausted (no more numbers available).
    #[error("Fiscal sequence exhausted: {0}")]
    FiscalSequenceExhausted(Uuid),

    /// No active CAI (Clave de Autorización de Impresión) found for the store.
    #[error("No active CAI for store: {0}")]
    NoActiveCai(Uuid),

    /// The CAI has expired.
    #[error("CAI expired for store: {0}")]
    CaiExpired(Uuid),

    /// The fiscal sequence range has been fully exhausted.
    #[error("Fiscal sequence range exhausted")]
    SequenceExhausted,

    // -------------------------------------------------------------------------
    // Sale reference errors
    // -------------------------------------------------------------------------
    /// The referenced sale was not found.
    #[error("Sale not found: {0}")]
    SaleNotFound(Uuid),

    /// The referenced sale is not in completed status.
    #[error("Sale is not completed: {0}")]
    SaleNotCompleted(Uuid),

    /// An invoice already exists for this sale.
    #[error("Invoice already exists for sale: {0}")]
    InvoiceAlreadyExistsForSale(Uuid),

    // -------------------------------------------------------------------------
    // Validation errors
    // -------------------------------------------------------------------------
    /// The provided invoice type is not recognized.
    #[error("Invalid invoice type")]
    InvalidInvoiceType,

    /// The provided invoice status is not recognized.
    #[error("Invalid invoice status")]
    InvalidInvoiceStatus,

    /// The provided tax type is not recognized.
    #[error("Invalid tax type")]
    InvalidTaxType,

    /// The provided tax rate value is invalid.
    #[error("Invalid tax rate")]
    InvalidTaxRate,

    /// A credit note requires a reference to the original invoice.
    #[error("Original invoice is required for credit notes")]
    OriginalInvoiceRequired,

    /// The credit note amount exceeds the original invoice total.
    #[error("Credit note amount exceeds the original invoice total")]
    CreditNoteExceedsOriginal,

    /// The provided tax applies-to scope is not recognized.
    #[error("Invalid tax applies-to scope")]
    InvalidTaxAppliesTo,

    // -------------------------------------------------------------------------
    // Database and system errors
    // -------------------------------------------------------------------------
    /// An error occurred while recording an audit entry.
    #[error("Audit error: {0}")]
    AuditError(String),

    /// A database error occurred during the operation.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// The requested functionality is not yet implemented.
    #[error("Not implemented")]
    NotImplemented,
}
