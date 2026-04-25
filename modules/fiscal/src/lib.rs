//! # Fiscal Module
//!
//! Fiscal management for a multi-store POS and e-commerce system (Honduras).
//!
//! This module provides:
//! - **Invoice Management**: Generation, voiding, and listing of fiscal invoices
//! - **Tax Calculation**: Honduras ISV (15% and 18%) tax computation
//! - **Fiscal Sequences**: CAI-based sequential invoice numbering
//! - **Tax Rate Configuration**: Store-level tax rate management
//!
//! ## Architecture
//!
//! The module follows hexagonal/clean architecture with three layers:
//!
//! - **Domain Layer**: Core business logic, entities, value objects, repository traits
//! - **Application Layer**: Use cases, DTOs, orchestration
//! - **Infrastructure Layer**: PostgreSQL repository implementations
//!
//! ## Workflows
//!
//! ### Invoice Workflow
//! ```text
//! Draft → Emitted (with CAI and fiscal sequence)
//!      ↘ Cancelled
//! Emitted → Voided (with reason)
//! ```

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

// =============================================================================
// Public API - Re-exports for convenient access
// =============================================================================

/// Error type for all fiscal operations
pub use error::FiscalError;

// -----------------------------------------------------------------------------
// Domain Layer - Value Objects
// -----------------------------------------------------------------------------

// ID value objects - UUID v7 based identifiers for temporal ordering
pub use domain::value_objects::FiscalSequenceId;
pub use domain::value_objects::InvoiceId;
pub use domain::value_objects::InvoiceLineId;
pub use domain::value_objects::TaxRateId;

// Enum value objects
pub use domain::value_objects::InvoiceStatus;
pub use domain::value_objects::InvoiceType;
pub use domain::value_objects::TaxAppliesTo;
pub use domain::value_objects::TaxType;

// -----------------------------------------------------------------------------
// Domain Layer - Entities
// -----------------------------------------------------------------------------

pub use domain::entities::FiscalSequence;
pub use domain::entities::Invoice;
pub use domain::entities::InvoiceLine;
pub use domain::entities::TaxRate;

// -----------------------------------------------------------------------------
// Domain Layer - Repository Traits
// -----------------------------------------------------------------------------

pub use domain::repositories::FiscalSequenceRepository;
pub use domain::repositories::InvoiceFilter;
pub use domain::repositories::InvoiceRepository;
pub use domain::repositories::TaxRateRepository;

// -----------------------------------------------------------------------------
// Infrastructure Layer - PostgreSQL Repository Implementations
// -----------------------------------------------------------------------------

pub use infrastructure::persistence::PgFiscalSequenceRepository;
pub use infrastructure::persistence::PgInvoiceRepository;
pub use infrastructure::persistence::PgTaxRateRepository;

// -----------------------------------------------------------------------------
// Application Layer - DTOs
// -----------------------------------------------------------------------------

// Invoice DTOs
pub use application::dtos::CalculateTaxCommand;
pub use application::dtos::FiscalReportCommand;
pub use application::dtos::FiscalReportResponse;
pub use application::dtos::GenerateInvoiceCommand;
pub use application::dtos::InvoiceLineResponse;
pub use application::dtos::InvoiceListResponse;
pub use application::dtos::InvoiceResponse;
pub use application::dtos::InvoiceSummaryResponse;
pub use application::dtos::ListInvoicesQuery;
pub use application::dtos::TaxCalculationItem;
pub use application::dtos::TaxCalculationResponse;
pub use application::dtos::TaxCalculationResultItem;
pub use application::dtos::VoidInvoiceCommand;

// Tax Rate DTOs
pub use application::dtos::CreateTaxRateCommand;
pub use application::dtos::TaxRateResponse;
pub use application::dtos::UpdateTaxRateCommand;

// -----------------------------------------------------------------------------
// Application Layer - Use Cases
// -----------------------------------------------------------------------------

// Invoice Use Cases
pub use application::use_cases::CalculateTaxUseCase;
pub use application::use_cases::FiscalReportUseCase;
pub use application::use_cases::GenerateInvoiceUseCase;
pub use application::use_cases::GetInvoiceUseCase;
pub use application::use_cases::ListInvoicesUseCase;
pub use application::use_cases::VoidInvoiceUseCase;

// Tax Rate Use Cases
pub use application::use_cases::CreateTaxRateUseCase;
pub use application::use_cases::DeleteTaxRateUseCase;
pub use application::use_cases::GetTaxRateUseCase;
pub use application::use_cases::ListTaxRatesUseCase;
pub use application::use_cases::UpdateTaxRateUseCase;
