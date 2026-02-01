// Sales handlers module
//
// This module organizes all sales-related HTTP handlers by domain:
// - customers: Customer CRUD and status management
// - shifts: Cashier shift lifecycle operations
// - pos: Point-of-Sale transaction operations

pub mod cart;
pub mod credit_notes;
pub mod customers;
pub mod pos;
pub mod shifts;

// Re-export all handlers for easy access
pub use cart::*;
pub use credit_notes::*;
pub use customers::*;
pub use pos::*;
pub use shifts::*;
