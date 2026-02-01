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

pub mod application;
pub mod domain;
pub mod infrastructure;

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

// -----------------------------------------------------------------------------
// Infrastructure Layer - PostgreSQL Repository Implementations
// -----------------------------------------------------------------------------

pub use infrastructure::persistence::PgCartRepository;
pub use infrastructure::persistence::PgCreditNoteRepository;
pub use infrastructure::persistence::PgCustomerRepository;
pub use infrastructure::persistence::PgSaleRepository;
pub use infrastructure::persistence::PgShiftRepository;

// -----------------------------------------------------------------------------
// Application Layer - DTOs
// -----------------------------------------------------------------------------

// Customer DTOs
pub use application::dtos::AddressInput;
pub use application::dtos::AddressResponse;
pub use application::dtos::CreateCustomerCommand;
pub use application::dtos::CustomerListResponse;
pub use application::dtos::CustomerResponse;
pub use application::dtos::ListCustomersQuery;
pub use application::dtos::UpdateCustomerCommand;

// Shift DTOs
pub use application::dtos::CashMovementCommand;
pub use application::dtos::CloseShiftCommand;
pub use application::dtos::ListShiftsQuery;
pub use application::dtos::OpenShiftCommand;
pub use application::dtos::PaymentBreakdownItem;
pub use application::dtos::SalesBreakdown;
pub use application::dtos::ShiftListResponse;
pub use application::dtos::ShiftReportResponse;
pub use application::dtos::ShiftResponse;

// Sale DTOs
pub use application::dtos::AddSaleItemCommand;
pub use application::dtos::ApplyDiscountCommand;
pub use application::dtos::CreatePosSaleCommand;
pub use application::dtos::ListSalesQuery;
pub use application::dtos::PaymentResponse;
pub use application::dtos::ProcessPaymentCommand;
pub use application::dtos::SaleDetailResponse;
pub use application::dtos::SaleItemResponse;
pub use application::dtos::SaleListResponse;
pub use application::dtos::SaleResponse;
pub use application::dtos::UpdateSaleItemCommand;
pub use application::dtos::VoidSaleCommand;

// -----------------------------------------------------------------------------
// Application Layer - Use Cases
// -----------------------------------------------------------------------------

// Customer Use Cases
pub use application::use_cases::CreateCustomerUseCase;
pub use application::use_cases::GetCustomerUseCase;
pub use application::use_cases::ListCustomersUseCase;
pub use application::use_cases::ToggleCustomerStatusUseCase;
pub use application::use_cases::UpdateCustomerUseCase;

// Shift Use Cases
pub use application::use_cases::CloseShiftUseCase;
pub use application::use_cases::GetCurrentShiftUseCase;
pub use application::use_cases::GetShiftReportUseCase;
pub use application::use_cases::ListShiftsUseCase;
pub use application::use_cases::OpenShiftUseCase;
pub use application::use_cases::RecordCashMovementUseCase;

// POS Sale Use Cases
pub use application::use_cases::AddSaleItemUseCase;
pub use application::use_cases::ApplyDiscountUseCase;
pub use application::use_cases::CompleteSaleUseCase;
pub use application::use_cases::CreatePosSaleUseCase;
pub use application::use_cases::GetSaleUseCase;
pub use application::use_cases::ListSalesUseCase;
pub use application::use_cases::ProcessPaymentUseCase;
pub use application::use_cases::RemoveSaleItemUseCase;
pub use application::use_cases::UpdateSaleItemUseCase;
pub use application::use_cases::VoidSaleUseCase;
