use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum CashManagementError {
    #[error("Bank account not found: {0}")]
    BankAccountNotFound(Uuid),

    #[error("Bank transaction not found: {0}")]
    BankTransactionNotFound(Uuid),

    #[error("Cash deposit not found: {0}")]
    CashDepositNotFound(Uuid),

    #[error("Bank reconciliation not found: {0}")]
    ReconciliationNotFound(Uuid),

    #[error("Duplicate bank account number: {0}")]
    DuplicateAccountNumber(String),

    #[error("Cashier shift not found: {0}")]
    ShiftNotFound(Uuid),

    #[error("Cashier shift {0} is not closed; cannot deposit yet")]
    ShiftNotClosed(Uuid),

    #[error("Bank reconciliation date range invalid: starts must be < ends")]
    InvalidReconciliationRange,

    #[error("Cannot transition deposit from {from} to {to}")]
    InvalidDepositTransition { from: String, to: String },

    #[error("Cannot transition reconciliation from {from} to {to}")]
    InvalidReconciliationTransition { from: String, to: String },

    #[error(
        "Reconciliation does not balance: statement={statement} book={book} difference={difference}"
    )]
    ReconciliationUnbalanced {
        statement: rust_decimal::Decimal,
        book: rust_decimal::Decimal,
        difference: rust_decimal::Decimal,
    },

    #[error("Bank transaction is already linked to another deposit")]
    TransactionAlreadyLinked,

    #[error("Bank transaction belongs to a different account than the deposit")]
    TransactionAccountMismatch,

    #[error("Transaction amount mismatch: deposit={deposit} transaction={transaction}")]
    TransactionAmountMismatch {
        deposit: rust_decimal::Decimal,
        transaction: rust_decimal::Decimal,
    },

    #[error("Negative amount is not allowed")]
    NegativeAmount,

    #[error("Optimistic lock conflict on bank account {0}")]
    AccountVersionConflict(Uuid),

    #[error("Invalid bank transaction type: {0}")]
    InvalidTransactionType(String),

    #[error("Invalid bank account type: {0}")]
    InvalidAccountType(String),

    #[error("Invalid deposit status: {0}")]
    InvalidDepositStatus(String),

    #[error("Invalid reconciliation status: {0}")]
    InvalidReconciliationStatus(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Subscriber error: {0}")]
    Subscriber(String),
}
