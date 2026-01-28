//! Purchasing module error types.
//!
//! This module defines all error types that can occur during purchasing operations.
//! Errors are categorized by domain area (vendors, purchase orders, goods receipts).

use thiserror::Error;
use uuid::Uuid;

/// Error type for all purchasing module operations.
///
/// This enum covers all possible error conditions that can occur when working
/// with the purchasing module, including validation errors, not-found errors,
/// and workflow constraint violations.
#[derive(Debug, Error)]
pub enum PurchasingError {
    // -------------------------------------------------------------------------
    // Vendor errors
    // -------------------------------------------------------------------------

    /// The requested vendor was not found in the database.
    #[error("Vendor not found: {0}")]
    VendorNotFound(Uuid),

    /// A vendor with the given code already exists.
    #[error("Vendor code '{0}' already exists")]
    DuplicateVendorCode(String),

    /// A vendor with the given tax ID already exists.
    #[error("Vendor tax ID '{0}' already exists")]
    DuplicateVendorTaxId(String),

    /// The vendor is not active.
    #[error("Vendor is not active: {0}")]
    VendorNotActive(Uuid),

    // -------------------------------------------------------------------------
    // Purchase Order errors
    // -------------------------------------------------------------------------

    /// The requested purchase order was not found in the database.
    #[error("Purchase order not found: {0}")]
    PurchaseOrderNotFound(Uuid),

    /// A purchase order with the given order number already exists in the store.
    #[error("Purchase order number '{0}' already exists")]
    DuplicateOrderNumber(String),

    /// Cannot modify a purchase order that is not in Draft status.
    #[error("Cannot modify purchase order: not in draft status")]
    OrderNotEditable,

    /// Cannot submit an empty purchase order.
    #[error("Purchase order has no items")]
    EmptyPurchaseOrder,

    /// User cannot approve their own purchase order.
    #[error("User cannot approve their own purchase order")]
    CannotApproveSelfCreatedOrder,

    /// The purchase order has already been cancelled.
    #[error("Purchase order has already been cancelled")]
    OrderAlreadyCancelled,

    /// The purchase order has already been closed.
    #[error("Purchase order has already been closed")]
    OrderAlreadyClosed,

    /// Cannot receive goods for a purchase order that is not approved.
    #[error("Cannot receive goods: purchase order not approved")]
    OrderNotApproved,

    /// Cannot cancel an order that has received goods.
    #[error("Cannot cancel: purchase order has received goods")]
    OrderHasReceivedGoods,

    // -------------------------------------------------------------------------
    // Purchase Order Item errors
    // -------------------------------------------------------------------------

    /// The requested purchase order item was not found.
    #[error("Purchase order item not found: {0}")]
    PurchaseOrderItemNotFound(Uuid),

    /// Quantity ordered must be greater than zero.
    #[error("Quantity ordered must be positive")]
    InvalidQuantityOrdered,

    /// Unit cost must be non-negative.
    #[error("Unit cost must be non-negative")]
    InvalidUnitCost,

    /// Cannot receive more than ordered quantity.
    #[error("Cannot receive more than ordered quantity")]
    ExceedsOrderedQuantity,

    // -------------------------------------------------------------------------
    // Goods Receipt errors
    // -------------------------------------------------------------------------

    /// The requested goods receipt was not found in the database.
    #[error("Goods receipt not found: {0}")]
    GoodsReceiptNotFound(Uuid),

    /// A goods receipt with the given receipt number already exists.
    #[error("Goods receipt number '{0}' already exists")]
    DuplicateReceiptNumber(String),

    /// Cannot modify a goods receipt that is not in Draft status.
    #[error("Cannot modify goods receipt: not in draft status")]
    ReceiptNotEditable,

    /// Cannot confirm an empty goods receipt.
    #[error("Goods receipt has no items")]
    EmptyGoodsReceipt,

    /// The goods receipt has already been confirmed.
    #[error("Goods receipt has already been confirmed")]
    ReceiptAlreadyConfirmed,

    /// The goods receipt has already been cancelled.
    #[error("Goods receipt has already been cancelled")]
    ReceiptAlreadyCancelled,

    // -------------------------------------------------------------------------
    // Goods Receipt Item errors
    // -------------------------------------------------------------------------

    /// The requested goods receipt item was not found.
    #[error("Goods receipt item not found: {0}")]
    GoodsReceiptItemNotFound(Uuid),

    /// Quantity received must be greater than zero.
    #[error("Quantity received must be positive")]
    InvalidQuantityReceived,

    // -------------------------------------------------------------------------
    // Workflow errors
    // -------------------------------------------------------------------------

    /// The requested status transition is not valid for the current state.
    #[error("Invalid status transition")]
    InvalidStatusTransition,

    // -------------------------------------------------------------------------
    // Validation errors
    // -------------------------------------------------------------------------

    /// Currency code must be exactly 3 uppercase letters (ISO 4217 format).
    #[error("Invalid currency code: must be 3 uppercase letters (ISO 4217)")]
    InvalidCurrency,

    /// The provided unit of measure is not recognized.
    #[error("Invalid unit of measure")]
    InvalidUnitOfMeasure,

    /// The provided purchase order status is not recognized.
    #[error("Invalid purchase order status")]
    InvalidPurchaseOrderStatus,

    /// The provided goods receipt status is not recognized.
    #[error("Invalid goods receipt status")]
    InvalidGoodsReceiptStatus,

    /// Product not found.
    #[error("Product not found: {0}")]
    ProductNotFound(Uuid),

    /// Store not found.
    #[error("Store not found: {0}")]
    StoreNotFound(Uuid),

    // -------------------------------------------------------------------------
    // Database errors
    // -------------------------------------------------------------------------

    /// A database error occurred during the operation.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // -------------------------------------------------------------------------
    // General errors
    // -------------------------------------------------------------------------

    /// The requested functionality is not yet implemented.
    #[error("Not implemented")]
    NotImplemented,
}
