//! DTOs (Data Transfer Objects) for the sales module.
//!
//! Contains commands (inputs) and responses (outputs) for all operations.

pub mod customer;
pub mod sale;
pub mod shift;

pub use customer::*;
pub use sale::*;
pub use shift::*;
