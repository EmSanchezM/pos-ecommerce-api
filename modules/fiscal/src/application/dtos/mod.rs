//! DTOs (Data Transfer Objects) for the fiscal module.
//!
//! Contains commands (inputs) and responses (outputs) for all operations.

pub mod invoice;
pub mod tax_rate;

pub use invoice::*;
pub use tax_rate::*;
