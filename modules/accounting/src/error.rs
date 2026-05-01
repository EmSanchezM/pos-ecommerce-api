use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AccountingError {
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    #[error("Account code not found: {0}")]
    AccountCodeNotFound(String),

    #[error("Period not found: {0}")]
    PeriodNotFound(Uuid),

    #[error("Journal entry not found: {0}")]
    JournalEntryNotFound(Uuid),

    #[error("Duplicate account code: {0}")]
    DuplicateAccountCode(String),

    #[error("Period is closed: {0}")]
    PeriodClosed(Uuid),

    #[error("Cannot post into a non-open period")]
    PeriodNotOpen,

    #[error("Period dates invalid: starts_at must be < ends_at")]
    InvalidPeriodRange,

    #[error("Journal entry must have at least two lines")]
    NotEnoughLines,

    #[error("Journal entry is unbalanced: debits={debits} credits={credits}")]
    Unbalanced { debits: Decimal, credits: Decimal },

    #[error("Journal line must have either debit or credit set, not both/neither")]
    InvalidLineAmounts,

    #[error("Journal line amounts must be non-negative")]
    NegativeAmount,

    #[error("Invalid account type: {0}")]
    InvalidAccountType(String),

    #[error("Invalid period status: {0}")]
    InvalidPeriodStatus(String),

    #[error("Invalid journal entry status: {0}")]
    InvalidJournalEntryStatus(String),

    #[error("Cannot transition entry from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Subscriber error: {0}")]
    Subscriber(String),
}
