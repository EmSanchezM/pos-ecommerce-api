//! Repository traits for the sales domain.
//!
//! This module defines the repository interfaces (traits) for persisting
//! sales domain entities. Implementations are in the infrastructure layer.

mod cart_repository;
mod credit_note_repository;
mod customer_repository;
mod sale_repository;
mod shift_repository;

pub use cart_repository::{CartFilter, CartRepository};
pub use credit_note_repository::{CreditNoteFilter, CreditNoteRepository};
pub use customer_repository::{CustomerFilter, CustomerRepository};
pub use sale_repository::{SaleFilter, SaleRepository};
pub use shift_repository::{ShiftFilter, ShiftRepository};
