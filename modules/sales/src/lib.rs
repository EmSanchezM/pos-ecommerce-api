//! # Sales Module
//!
//! Comprehensive sales management for a multi-store POS and e-commerce system.
//!
//! This module provides:
//! - **Customer Management**: CRUD operations for customers
//! - **POS Sales**: Point of sale transactions with payments
//! - **E-commerce Orders**: Online order processing with cart management
//! - **Cashier Shifts**: Shift management for POS terminals
//! - **Returns/Credit Notes**: Product returns with approval workflow
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
//! ### POS Sale Workflow
//! ```text
//! Draft → Completed (with payment and invoice)
//!      ↘ Voided
//! Completed → Returned (via Credit Note)
//! ```
//!
//! ### E-commerce Order Workflow
//! ```text
//! PendingPayment → Paid → Processing → Shipped → Delivered
//!              ↘ PaymentFailed     ↘ Cancelled
//! Delivered → Returned (via Credit Note)
//! ```
//!
//! ### Credit Note Workflow
//! ```text
//! Draft → Pending → Approved → Applied
//!      ↘ Cancelled  ↘ Cancelled
//! ```

pub mod domain;

mod error;

// =============================================================================
// Public API - Re-exports for convenient access
// =============================================================================

/// Error type for all sales operations
pub use error::SalesError;

// -----------------------------------------------------------------------------
// Domain Layer - Value Objects
// -----------------------------------------------------------------------------

// ID value objects - UUID v7 based identifiers for temporal ordering
pub use domain::value_objects::CartId;
pub use domain::value_objects::CartItemId;
pub use domain::value_objects::CreditNoteId;
pub use domain::value_objects::CreditNoteItemId;
pub use domain::value_objects::CustomerId;
pub use domain::value_objects::PaymentId;
pub use domain::value_objects::SaleId;
pub use domain::value_objects::SaleItemId;
pub use domain::value_objects::ShiftId;

// Enum value objects
pub use domain::value_objects::CreditNoteStatus;
pub use domain::value_objects::CustomerType;
pub use domain::value_objects::DiscountType;
pub use domain::value_objects::OrderStatus;
pub use domain::value_objects::PaymentMethod;
pub use domain::value_objects::PaymentStatus;
pub use domain::value_objects::ReturnReason;
pub use domain::value_objects::ReturnType;
pub use domain::value_objects::SaleStatus;
pub use domain::value_objects::SaleType;
pub use domain::value_objects::ShiftStatus;

// -----------------------------------------------------------------------------
// Domain Layer - Entities
// -----------------------------------------------------------------------------

pub use domain::entities::Address;
pub use domain::entities::Cart;
pub use domain::entities::CartItem;
pub use domain::entities::CashierShift;
pub use domain::entities::CreditNote;
pub use domain::entities::CreditNoteItem;
pub use domain::entities::Customer;
pub use domain::entities::Payment;
pub use domain::entities::Sale;
pub use domain::entities::SaleItem;

// -----------------------------------------------------------------------------
// Domain Layer - Repository Traits
// -----------------------------------------------------------------------------

pub use domain::repositories::CartFilter;
pub use domain::repositories::CartRepository;
pub use domain::repositories::CreditNoteFilter;
pub use domain::repositories::CreditNoteRepository;
pub use domain::repositories::CustomerFilter;
pub use domain::repositories::CustomerRepository;
pub use domain::repositories::SaleFilter;
pub use domain::repositories::SaleRepository;
pub use domain::repositories::ShiftFilter;
pub use domain::repositories::ShiftRepository;
