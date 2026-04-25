//! Use cases for the fiscal module.
//!
//! Organized by domain area:
//! - invoice: Invoice generation, voiding, listing, and tax calculation
//! - tax_rate: Tax rate CRUD operations

pub mod invoice;
pub mod tax_rate;

pub use invoice::*;
pub use tax_rate::*;
