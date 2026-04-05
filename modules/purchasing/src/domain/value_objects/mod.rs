//! Domain value objects for purchasing management.
//!
//! Value objects are immutable objects defined by their attributes rather than identity.
//! They encapsulate validation rules and provide type safety.
//!
//! ## ID Value Objects
//!
//! All IDs use UUID v7 for temporal ordering:
//!
//! - [`VendorId`]: Vendor identifier
//! - [`PurchaseOrderId`]: Purchase order identifier
//! - [`PurchaseOrderItemId`]: Purchase order item identifier
//! - [`GoodsReceiptId`]: Goods receipt identifier
//! - [`GoodsReceiptItemId`]: Goods receipt item identifier
//!
//! ## Enum Value Objects
//!
//! - [`PurchaseOrderStatus`]: Purchase order workflow states
//! - [`GoodsReceiptStatus`]: Goods receipt workflow states

// ID value objects
mod goods_receipt_id;
mod goods_receipt_item_id;
mod purchase_order_id;
mod purchase_order_item_id;
mod vendor_id;

// Enum value objects
mod goods_receipt_status;
mod purchase_order_status;

// Re-exports - ID value objects
pub use goods_receipt_id::GoodsReceiptId;
pub use goods_receipt_item_id::GoodsReceiptItemId;
pub use purchase_order_id::PurchaseOrderId;
pub use purchase_order_item_id::PurchaseOrderItemId;
pub use vendor_id::VendorId;

// Re-exports - Enum value objects
pub use goods_receipt_status::GoodsReceiptStatus;
pub use purchase_order_status::PurchaseOrderStatus;
