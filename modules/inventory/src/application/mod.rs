//! Application layer for the inventory module.
//!
//! This layer contains use cases (application services) and DTOs that orchestrate
//! domain operations and handle input/output transformation.
//!
//! ## Use Cases
//!
//! Use cases implement specific business operations by coordinating domain entities
//! and repositories. See [`use_cases`] for available operations.
//!
//! ## DTOs
//!
//! Data Transfer Objects for API boundaries:
//!
//! - **Commands**: Input DTOs for operations (e.g., `CreateProductCommand`)
//! - **Responses**: Output DTOs for API responses (e.g., `ProductResponse`)
//!
//! See [`dtos`] for all available DTOs.

pub mod dtos;
pub mod use_cases;
