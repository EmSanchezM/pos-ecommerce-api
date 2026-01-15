//! Application DTOs - Commands and Responses.
//!
//! This module contains Data Transfer Objects used at the application boundary:
//!
//! - **Commands**: Input DTOs that represent requests to perform operations
//! - **Responses**: Output DTOs that represent the results of operations
//!
//! DTOs use primitive types (String, Uuid, Decimal) rather than domain value objects
//! to keep the application boundary clean and allow validation in use cases.

pub mod commands;
pub mod responses;

// Re-export commonly used types
pub use commands::*;
pub use responses::*;
