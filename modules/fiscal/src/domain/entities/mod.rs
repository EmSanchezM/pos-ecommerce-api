//! Domain entities for the fiscal module.
//!
//! This module contains all business entities used in the fiscal module,
//! including invoices, invoice lines, tax rates, and fiscal sequences.

mod fiscal_sequence;
mod invoice;
mod invoice_line;
mod tax_rate;

pub use fiscal_sequence::FiscalSequence;
pub use invoice::Invoice;
pub use invoice_line::InvoiceLine;
pub use tax_rate::TaxRate;
