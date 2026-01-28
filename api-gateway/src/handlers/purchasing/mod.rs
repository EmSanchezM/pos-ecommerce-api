// Purchasing handlers module
//
// This module organizes all purchasing-related HTTP handlers by domain:
// - vendors: Vendor CRUD and status management
// - purchase_orders: Purchase order workflow operations
// - goods_receipts: Goods receipt operations

pub mod goods_receipts;
pub mod purchase_orders;
pub mod vendors;

// Re-export all handlers for easy access
pub use goods_receipts::*;
pub use purchase_orders::*;
pub use vendors::*;
