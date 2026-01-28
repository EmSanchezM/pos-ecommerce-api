//! # Purchasing Module
//!
//! Comprehensive purchasing management for a multi-store POS and e-commerce system.
//!
//! This module provides:
//! - **Vendor Management**: CRUD operations for suppliers/vendors
//! - **Purchase Orders**: Order creation with approval workflow
//! - **Goods Receipts**: Receiving merchandise and updating inventory
//!
//! ## Architecture
//!
//! The module follows hexagonal/clean architecture with three layers:
//!
//! - **Domain Layer**: Core business logic, entities, value objects, repository traits
//! - **Application Layer**: Use cases, DTOs, orchestration
//! - **Infrastructure Layer**: PostgreSQL repository implementations
//!
//! ## Workflow
//!
//! ### Purchase Order Workflow
//! ```text
//! Draft → Submitted → Approved → PartiallyReceived/Received → Closed
//!                  ↘ Draft (rejected)
//! Draft/Submitted → Cancelled
//! ```
//!
//! ### Goods Receipt Workflow
//! ```text
//! Draft → Confirmed (affects inventory)
//!      ↘ Cancelled
//! ```

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

// =============================================================================
// Public API - Re-exports for convenient access
// =============================================================================

/// Error type for all purchasing operations
pub use error::PurchasingError;

// -----------------------------------------------------------------------------
// Domain Layer - Value Objects
// -----------------------------------------------------------------------------

// ID value objects - UUID v7 based identifiers for temporal ordering
pub use domain::value_objects::GoodsReceiptId;
pub use domain::value_objects::GoodsReceiptItemId;
pub use domain::value_objects::PurchaseOrderId;
pub use domain::value_objects::PurchaseOrderItemId;
pub use domain::value_objects::VendorId;

// Enum value objects
pub use domain::value_objects::GoodsReceiptStatus;
pub use domain::value_objects::PurchaseOrderStatus;

// -----------------------------------------------------------------------------
// Domain Layer - Entities
// -----------------------------------------------------------------------------

pub use domain::entities::GoodsReceipt;
pub use domain::entities::GoodsReceiptItem;
pub use domain::entities::PurchaseOrder;
pub use domain::entities::PurchaseOrderItem;
pub use domain::entities::Vendor;

// -----------------------------------------------------------------------------
// Domain Layer - Repository Traits
// -----------------------------------------------------------------------------

pub use domain::repositories::GoodsReceiptFilter;
pub use domain::repositories::GoodsReceiptRepository;
pub use domain::repositories::PurchaseOrderFilter;
pub use domain::repositories::PurchaseOrderRepository;
pub use domain::repositories::VendorFilter;
pub use domain::repositories::VendorRepository;

// -----------------------------------------------------------------------------
// Infrastructure Layer - PostgreSQL Repositories
// -----------------------------------------------------------------------------

pub use infrastructure::persistence::PgGoodsReceiptRepository;
pub use infrastructure::persistence::PgPurchaseOrderRepository;
pub use infrastructure::persistence::PgVendorRepository;

// -----------------------------------------------------------------------------
// Application Layer - DTOs
// -----------------------------------------------------------------------------

// Command DTOs
pub use application::dtos::commands::AddOrderItemCommand;
pub use application::dtos::commands::CancelOrderCommand;
pub use application::dtos::commands::CreateGoodsReceiptCommand;
pub use application::dtos::commands::CreateGoodsReceiptItemCommand;
pub use application::dtos::commands::CreatePurchaseOrderCommand;
pub use application::dtos::commands::CreatePurchaseOrderItemCommand;
pub use application::dtos::commands::CreateVendorCommand;
pub use application::dtos::commands::RejectOrderCommand;
pub use application::dtos::commands::UpdateGoodsReceiptCommand;
pub use application::dtos::commands::UpdateOrderItemCommand;
pub use application::dtos::commands::UpdatePurchaseOrderCommand;
pub use application::dtos::commands::UpdateVendorCommand;

// Response DTOs
pub use application::dtos::responses::GoodsReceiptDetailResponse;
pub use application::dtos::responses::GoodsReceiptItemResponse;
pub use application::dtos::responses::GoodsReceiptResponse;
pub use application::dtos::responses::PurchaseOrderDetailResponse;
pub use application::dtos::responses::PurchaseOrderItemResponse;
pub use application::dtos::responses::PurchaseOrderResponse;
pub use application::dtos::responses::VendorResponse;

// -----------------------------------------------------------------------------
// Application Layer - Use Cases
// -----------------------------------------------------------------------------

// Vendor Use Cases
pub use application::use_cases::CreateVendorUseCase;
pub use application::use_cases::GetVendorUseCase;
pub use application::use_cases::ListVendorsQuery;
pub use application::use_cases::ListVendorsUseCase;
pub use application::use_cases::ToggleVendorStatusUseCase;
pub use application::use_cases::UpdateVendorUseCase;

// Purchase Order Use Cases
pub use application::use_cases::ApprovePurchaseOrderUseCase;
pub use application::use_cases::CancelPurchaseOrderUseCase;
pub use application::use_cases::ClosePurchaseOrderUseCase;
pub use application::use_cases::CreatePurchaseOrderUseCase;
pub use application::use_cases::GetPurchaseOrderUseCase;
pub use application::use_cases::ListPurchaseOrdersQuery;
pub use application::use_cases::ListPurchaseOrdersUseCase;
pub use application::use_cases::RejectPurchaseOrderUseCase;
pub use application::use_cases::SubmitPurchaseOrderUseCase;

// Goods Receipt Use Cases
pub use application::use_cases::CancelGoodsReceiptUseCase;
pub use application::use_cases::ConfirmGoodsReceiptUseCase;
pub use application::use_cases::CreateGoodsReceiptUseCase;
pub use application::use_cases::GetGoodsReceiptUseCase;
pub use application::use_cases::ListGoodsReceiptsQuery;
pub use application::use_cases::ListGoodsReceiptsUseCase;
