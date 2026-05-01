//! Payments module error types.

use thiserror::Error;
use uuid::Uuid;

/// Error type for all payments module operations.
#[derive(Debug, Error)]
pub enum PaymentsError {
    // -------------------------------------------------------------------------
    // Gateway errors
    // -------------------------------------------------------------------------
    #[error("Payment gateway not found: {0}")]
    GatewayNotFound(Uuid),

    #[error("No default gateway configured for store: {0}")]
    NoDefaultGateway(Uuid),

    #[error("Gateway is not active: {0}")]
    GatewayNotActive(Uuid),

    #[error("Gateway name '{0}' already exists for this store")]
    DuplicateGatewayName(String),

    #[error("Invalid gateway type")]
    InvalidGatewayType,

    // -------------------------------------------------------------------------
    // Transaction errors
    // -------------------------------------------------------------------------
    #[error("Transaction not found: {0}")]
    TransactionNotFound(Uuid),

    #[error("Duplicate idempotency key: {0}")]
    DuplicateIdempotencyKey(String),

    #[error("Transaction already processed: {0}")]
    TransactionAlreadyProcessed(Uuid),

    #[error("Refund exceeds original amount")]
    RefundExceedsOriginal,

    #[error("Transaction cannot be refunded in current status")]
    CannotRefundTransaction,

    #[error("Payment processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Invalid amount: must be positive")]
    InvalidAmount,

    #[error("Invalid transaction type")]
    InvalidTransactionType,

    #[error("Invalid transaction status")]
    InvalidTransactionStatus,

    // -------------------------------------------------------------------------
    // Manual confirmation errors
    // -------------------------------------------------------------------------
    #[error("Invalid manual payment kind")]
    InvalidManualPaymentKind,

    #[error("Transaction is not pending and cannot be confirmed/rejected: {0}")]
    TransactionNotPending(Uuid),

    #[error("Transaction has already been confirmed: {0}")]
    TransactionAlreadyConfirmed(Uuid),

    #[error("Transaction has already been rejected: {0}")]
    TransactionAlreadyRejected(Uuid),

    // -------------------------------------------------------------------------
    // Webhook errors
    // -------------------------------------------------------------------------
    #[error("Invalid webhook signature")]
    InvalidWebhookSignature,

    #[error("Unsupported payment method for gateway")]
    UnsupportedPaymentMethod,

    #[error("Unsupported currency for gateway")]
    UnsupportedCurrency,

    // -------------------------------------------------------------------------
    // Payout errors
    // -------------------------------------------------------------------------
    #[error("Payout not found: {0}")]
    PayoutNotFound(Uuid),

    #[error("Invalid payout status")]
    InvalidPayoutStatus,

    // -------------------------------------------------------------------------
    // Cross-module references
    // -------------------------------------------------------------------------
    #[error("Sale not found: {0}")]
    SaleNotFound(Uuid),

    // -------------------------------------------------------------------------
    // System errors
    // -------------------------------------------------------------------------
    #[error("Audit error: {0}")]
    AuditError(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Gateway communication error: {0}")]
    GatewayError(String),

    #[error("Not implemented")]
    NotImplemented,
}
