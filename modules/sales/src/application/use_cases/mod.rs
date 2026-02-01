//! Use cases for the sales module.
//!
//! Organized by domain area:
//! - customer: Customer management operations
//! - shift: Cashier shift operations
//! - pos: Point of Sale operations

pub mod customer;
pub mod pos;
pub mod shift;

pub use customer::*;
pub use pos::*;
pub use shift::*;
