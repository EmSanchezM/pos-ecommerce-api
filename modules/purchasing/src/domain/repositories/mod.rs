//! Repository traits for purchasing management.
//!
//! Repository traits define the contract for data persistence operations.
//! They are implemented by infrastructure layer components (e.g., PostgreSQL repositories).
//!
//! ## Available Repositories
//!
//! - [`VendorRepository`]: CRUD operations for vendors
//! - [`PurchaseOrderRepository`]: Purchase order persistence with items
//! - [`GoodsReceiptRepository`]: Goods receipt persistence

mod goods_receipt_repository;
mod purchase_order_repository;
mod vendor_repository;

pub use goods_receipt_repository::{GoodsReceiptFilter, GoodsReceiptRepository};
pub use purchase_order_repository::{PurchaseOrderFilter, PurchaseOrderRepository};
pub use vendor_repository::{VendorFilter, VendorRepository};
