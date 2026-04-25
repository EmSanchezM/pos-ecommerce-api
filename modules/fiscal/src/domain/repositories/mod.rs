//! Repository traits for the fiscal domain.
//!
//! This module defines the repository interfaces (traits) for persisting
//! fiscal domain entities. Implementations are in the infrastructure layer.

mod fiscal_sequence_repository;
mod invoice_repository;
mod tax_rate_repository;

pub use fiscal_sequence_repository::{FiscalSequenceRepository, NextSequenceResult};
pub use invoice_repository::{InvoiceFilter, InvoiceRepository};
pub use tax_rate_repository::TaxRateRepository;
