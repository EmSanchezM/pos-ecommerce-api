//! Use cases for the sales module.
//!
//! Organized by domain area:
//! - customer: Customer management operations
//! - shift: Cashier shift operations
//! - pos: Point of Sale operations

pub mod cart;
pub mod credit_note;
pub mod customer;
pub mod ecommerce;
pub mod pos;
pub mod promotion;
pub mod shift;

pub use cart::*;
pub use credit_note::*;
pub use customer::*;
pub use ecommerce::*;
pub use pos::*;
pub use promotion::*;
pub use shift::*;
