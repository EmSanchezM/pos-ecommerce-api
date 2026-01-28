//! Application layer for the purchasing module.
//!
//! This layer contains use cases (application services) and DTOs that orchestrate
//! domain operations and handle input/output transformation.
//!
//! ## DTOs
//!
//! Data Transfer Objects for API boundaries:
//!
//! - **Commands**: Input DTOs for operations (e.g., `CreateVendorCommand`)
//! - **Responses**: Output DTOs for API responses (e.g., `VendorResponse`)
//!
//! See [`dtos`] for all available DTOs.
//!
//! ## Use Cases
//!
//! Application services that orchestrate domain operations:
//!
//! - **Vendor**: `CreateVendorUseCase`, `UpdateVendorUseCase`, `GetVendorUseCase`, etc.
//! - **PurchaseOrder**: `CreatePurchaseOrderUseCase`, `SubmitPurchaseOrderUseCase`, etc.
//! - **GoodsReceipt**: `CreateGoodsReceiptUseCase`, `ConfirmGoodsReceiptUseCase`, etc.
//!
//! See [`use_cases`] for all available use cases.

pub mod dtos;
pub mod use_cases;
