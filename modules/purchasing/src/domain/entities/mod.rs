//! Domain entities for purchasing management.
//!
//! Entities are objects with distinct identity that persists over time.
//! They encapsulate business logic and maintain invariants.
//!
//! ## Available Entities
//!
//! - [`Vendor`]: Represents a supplier/vendor
//! - [`PurchaseOrder`]: Represents a purchase order document with workflow
//! - [`PurchaseOrderItem`]: Line item in a purchase order
//! - [`GoodsReceipt`]: Represents a goods receipt document
//! - [`GoodsReceiptItem`]: Line item in a goods receipt

mod vendor;
mod purchase_order;
mod purchase_order_item;
mod goods_receipt;
mod goods_receipt_item;

pub use vendor::Vendor;
pub use purchase_order::PurchaseOrder;
pub use purchase_order_item::PurchaseOrderItem;
pub use goods_receipt::GoodsReceipt;
pub use goods_receipt_item::GoodsReceiptItem;
