//! DTOs (Data Transfer Objects) for the sales module.
//!
//! Contains commands (inputs) and responses (outputs) for all operations.

pub mod cart;
pub mod credit_note;
pub mod customer;
pub mod sale;
pub mod shift;

pub use cart::*;
pub use credit_note::*;
pub use customer::*;
pub use sale::*;
pub use shift::*;
