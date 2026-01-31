//! Sales module error types.
//!
//! This module defines all error types that can occur during sales operations.
//! Errors are categorized by domain area (customers, sales, payments, carts, shifts, returns).

use thiserror::Error;
use uuid::Uuid;

/// Error type for all sales module operations.
///
/// This enum covers all possible error conditions that can occur when working
/// with the sales module, including validation errors, not-found errors,
/// and workflow constraint violations.
#[derive(Debug, Error)]
pub enum SalesError {
    // -------------------------------------------------------------------------
    // Customer errors
    // -------------------------------------------------------------------------

    /// The requested customer was not found in the database.
    #[error("Customer not found: {0}")]
    CustomerNotFound(Uuid),

    /// A customer with the given code already exists.
    #[error("Customer code '{0}' already exists")]
    DuplicateCustomerCode(String),

    /// A customer with the given email already exists.
    #[error("Customer email '{0}' already exists")]
    DuplicateCustomerEmail(String),

    /// The customer is not active.
    #[error("Customer is not active: {0}")]
    CustomerNotActive(Uuid),

    // -------------------------------------------------------------------------
    // Cashier Shift errors
    // -------------------------------------------------------------------------

    /// The requested cashier shift was not found.
    #[error("Cashier shift not found: {0}")]
    ShiftNotFound(Uuid),

    /// A shift is already open for this terminal.
    #[error("Terminal already has an open shift")]
    TerminalHasOpenShift,

    /// The cashier already has an open shift.
    #[error("Cashier already has an open shift")]
    CashierHasOpenShift,

    /// No open shift found for the terminal.
    #[error("No open shift found for terminal")]
    NoOpenShift,

    /// The shift is already closed.
    #[error("Shift is already closed")]
    ShiftAlreadyClosed,

    /// Opening balance must be non-negative.
    #[error("Opening balance must be non-negative")]
    InvalidOpeningBalance,

    // -------------------------------------------------------------------------
    // Sale errors
    // -------------------------------------------------------------------------

    /// The requested sale was not found in the database.
    #[error("Sale not found: {0}")]
    SaleNotFound(Uuid),

    /// A sale with the given sale number already exists in the store.
    #[error("Sale number '{0}' already exists")]
    DuplicateSaleNumber(String),

    /// Cannot modify a sale that is not in Draft status.
    #[error("Cannot modify sale: not in draft status")]
    SaleNotEditable,

    /// Cannot complete an empty sale.
    #[error("Sale has no items")]
    EmptySale,

    /// Sale is not fully paid.
    #[error("Sale is not fully paid")]
    SaleNotFullyPaid,

    /// The sale has already been completed.
    #[error("Sale has already been completed")]
    SaleAlreadyCompleted,

    /// The sale has already been voided.
    #[error("Sale has already been voided")]
    SaleAlreadyVoided,

    /// POS sale requires an open shift.
    #[error("POS sale requires an open shift")]
    PosRequiresOpenShift,

    /// POS sale requires a terminal.
    #[error("POS sale requires a terminal")]
    PosRequiresTerminal,

    /// POS sale requires a cashier.
    #[error("POS sale requires a cashier")]
    PosRequiresCashier,

    // -------------------------------------------------------------------------
    // Sale Item errors
    // -------------------------------------------------------------------------

    /// The requested sale item was not found.
    #[error("Sale item not found: {0}")]
    SaleItemNotFound(Uuid),

    /// Quantity must be greater than zero.
    #[error("Quantity must be positive")]
    InvalidQuantity,

    /// Unit price must be non-negative.
    #[error("Unit price must be non-negative")]
    InvalidUnitPrice,

    /// Insufficient stock for the product.
    #[error("Insufficient stock for product: {0}")]
    InsufficientStock(Uuid),

    // -------------------------------------------------------------------------
    // Payment errors
    // -------------------------------------------------------------------------

    /// The requested payment was not found.
    #[error("Payment not found: {0}")]
    PaymentNotFound(Uuid),

    /// Payment amount must be positive.
    #[error("Payment amount must be positive")]
    InvalidPaymentAmount,

    /// Payment exceeds remaining balance.
    #[error("Payment exceeds remaining balance")]
    PaymentExceedsBalance,

    /// The payment has already been refunded.
    #[error("Payment has already been refunded")]
    PaymentAlreadyRefunded,

    /// Cash payment requires amount tendered.
    #[error("Cash payment requires amount tendered")]
    CashRequiresAmountTendered,

    /// Amount tendered is less than payment amount.
    #[error("Amount tendered is less than payment amount")]
    InsufficientAmountTendered,

    // -------------------------------------------------------------------------
    // Cart errors
    // -------------------------------------------------------------------------

    /// The requested cart was not found.
    #[error("Cart not found: {0}")]
    CartNotFound(Uuid),

    /// The cart has expired.
    #[error("Cart has expired")]
    CartExpired,

    /// The cart is empty.
    #[error("Cart is empty")]
    EmptyCart,

    /// The requested cart item was not found.
    #[error("Cart item not found: {0}")]
    CartItemNotFound(Uuid),

    // -------------------------------------------------------------------------
    // Credit Note / Return errors
    // -------------------------------------------------------------------------

    /// The requested credit note was not found.
    #[error("Credit note not found: {0}")]
    CreditNoteNotFound(Uuid),

    /// A credit note with the given number already exists.
    #[error("Credit note number '{0}' already exists")]
    DuplicateCreditNoteNumber(String),

    /// Cannot modify credit note that is not in Draft status.
    #[error("Cannot modify credit note: not in draft status")]
    CreditNoteNotEditable,

    /// Cannot submit an empty credit note.
    #[error("Credit note has no items")]
    EmptyCreditNote,

    /// User cannot approve their own credit note.
    #[error("User cannot approve their own credit note")]
    CannotApproveSelfCreatedCreditNote,

    /// The credit note has already been approved.
    #[error("Credit note has already been approved")]
    CreditNoteAlreadyApproved,

    /// The credit note has already been cancelled.
    #[error("Credit note has already been cancelled")]
    CreditNoteAlreadyCancelled,

    /// The credit note has already been applied.
    #[error("Credit note has already been applied")]
    CreditNoteAlreadyApplied,

    /// Return quantity exceeds original sale quantity.
    #[error("Return quantity exceeds original sale quantity")]
    ReturnQuantityExceedsSaleQuantity,

    /// Cannot create return for incomplete sale.
    #[error("Cannot create return for incomplete sale")]
    SaleNotCompleted,

    // -------------------------------------------------------------------------
    // Order (E-commerce) errors
    // -------------------------------------------------------------------------

    /// Cannot cancel order that has been shipped.
    #[error("Cannot cancel order that has been shipped")]
    CannotCancelShippedOrder,

    /// Order has not been paid.
    #[error("Order has not been paid")]
    OrderNotPaid,

    /// Order is not in processing status.
    #[error("Order is not in processing status")]
    OrderNotProcessing,

    /// Order has not been shipped.
    #[error("Order has not been shipped")]
    OrderNotShipped,

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

    /// The provided sale status is not recognized.
    #[error("Invalid sale status")]
    InvalidSaleStatus,

    /// The provided order status is not recognized.
    #[error("Invalid order status")]
    InvalidOrderStatus,

    /// The provided payment method is not recognized.
    #[error("Invalid payment method")]
    InvalidPaymentMethod,

    /// The provided payment status is not recognized.
    #[error("Invalid payment status")]
    InvalidPaymentStatus,

    /// The provided discount type is not recognized.
    #[error("Invalid discount type")]
    InvalidDiscountType,

    /// The provided customer type is not recognized.
    #[error("Invalid customer type")]
    InvalidCustomerType,

    /// The provided return reason is not recognized.
    #[error("Invalid return reason")]
    InvalidReturnReason,

    /// The provided credit note status is not recognized.
    #[error("Invalid credit note status")]
    InvalidCreditNoteStatus,

    /// Discount percentage must be between 0 and 100.
    #[error("Discount percentage must be between 0 and 100")]
    InvalidDiscountPercentage,

    /// Tax rate must be non-negative.
    #[error("Tax rate must be non-negative")]
    InvalidTaxRate,

    /// Product not found.
    #[error("Product not found: {0}")]
    ProductNotFound(Uuid),

    /// Store not found.
    #[error("Store not found: {0}")]
    StoreNotFound(Uuid),

    /// Terminal not found.
    #[error("Terminal not found: {0}")]
    TerminalNotFound(Uuid),

    /// Terminal is not active.
    #[error("Terminal is not active: {0}")]
    TerminalNotActive(Uuid),

    /// No valid CAI available for terminal.
    #[error("No valid CAI available for terminal: {0}")]
    NoValidCai(Uuid),

    // -------------------------------------------------------------------------
    // Reservation errors
    // -------------------------------------------------------------------------

    /// Failed to create inventory reservation.
    #[error("Failed to create inventory reservation")]
    ReservationFailed,

    /// Failed to confirm inventory reservation.
    #[error("Failed to confirm inventory reservation")]
    ReservationConfirmFailed,

    /// Failed to cancel inventory reservation.
    #[error("Failed to cancel inventory reservation")]
    ReservationCancelFailed,

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
