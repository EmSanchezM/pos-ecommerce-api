//! Value objects for the fiscal domain.
//!
//! This module contains all value objects used in the fiscal module,
//! including IDs, status enums, and other immutable typed values.

// ID value objects
mod fiscal_sequence_id;
mod invoice_id;
mod invoice_line_id;
mod tax_rate_id;

// Enum value objects
mod invoice_status;
mod invoice_type;
mod tax_applies_to;
mod tax_type;

// Re-exports - IDs
pub use fiscal_sequence_id::FiscalSequenceId;
pub use invoice_id::InvoiceId;
pub use invoice_line_id::InvoiceLineId;
pub use tax_rate_id::TaxRateId;

// Re-exports - Enums
pub use invoice_status::InvoiceStatus;
pub use invoice_type::InvoiceType;
pub use tax_applies_to::TaxAppliesTo;
pub use tax_type::TaxType;
