//! Domain entities for the sales module.
//!
//! This module contains all business entities used in the sales module,
//! including customers, sales, payments, carts, shifts, and credit notes.

mod cart;
mod cart_item;
mod cashier_shift;
mod credit_note;
mod credit_note_item;
mod customer;
mod payment;
mod sale;
mod sale_item;

pub use cart::Cart;
pub use cart_item::CartItem;
pub use cashier_shift::CashierShift;
pub use credit_note::CreditNote;
pub use credit_note_item::CreditNoteItem;
pub use customer::{Address, Customer};
pub use payment::Payment;
pub use sale::Sale;
pub use sale_item::SaleItem;
