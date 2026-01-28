//! Use cases for the purchasing module.
//!
//! This module contains application services that orchestrate domain operations
//! for vendors, purchase orders, and goods receipts.

// -----------------------------------------------------------------------------
// Vendor Use Cases
// -----------------------------------------------------------------------------

mod create_vendor_use_case;
mod get_vendor_use_case;
mod list_vendors_use_case;
mod toggle_vendor_status_use_case;
mod update_vendor_use_case;

pub use create_vendor_use_case::CreateVendorUseCase;
pub use get_vendor_use_case::GetVendorUseCase;
pub use list_vendors_use_case::{ListVendorsQuery, ListVendorsUseCase};
pub use toggle_vendor_status_use_case::ToggleVendorStatusUseCase;
pub use update_vendor_use_case::UpdateVendorUseCase;

// -----------------------------------------------------------------------------
// Purchase Order Use Cases
// -----------------------------------------------------------------------------

mod approve_purchase_order_use_case;
mod cancel_purchase_order_use_case;
mod close_purchase_order_use_case;
mod create_purchase_order_use_case;
mod get_purchase_order_use_case;
mod list_purchase_orders_use_case;
mod reject_purchase_order_use_case;
mod submit_purchase_order_use_case;

pub use approve_purchase_order_use_case::ApprovePurchaseOrderUseCase;
pub use cancel_purchase_order_use_case::CancelPurchaseOrderUseCase;
pub use close_purchase_order_use_case::ClosePurchaseOrderUseCase;
pub use create_purchase_order_use_case::CreatePurchaseOrderUseCase;
pub use get_purchase_order_use_case::GetPurchaseOrderUseCase;
pub use list_purchase_orders_use_case::{ListPurchaseOrdersQuery, ListPurchaseOrdersUseCase};
pub use reject_purchase_order_use_case::RejectPurchaseOrderUseCase;
pub use submit_purchase_order_use_case::SubmitPurchaseOrderUseCase;

// -----------------------------------------------------------------------------
// Goods Receipt Use Cases
// -----------------------------------------------------------------------------

mod cancel_goods_receipt_use_case;
mod confirm_goods_receipt_use_case;
mod create_goods_receipt_use_case;
mod get_goods_receipt_use_case;
mod list_goods_receipts_use_case;

pub use cancel_goods_receipt_use_case::CancelGoodsReceiptUseCase;
pub use confirm_goods_receipt_use_case::ConfirmGoodsReceiptUseCase;
pub use create_goods_receipt_use_case::CreateGoodsReceiptUseCase;
pub use get_goods_receipt_use_case::GetGoodsReceiptUseCase;
pub use list_goods_receipts_use_case::{ListGoodsReceiptsQuery, ListGoodsReceiptsUseCase};
