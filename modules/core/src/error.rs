// Core module error types - full implementation in task 4

use thiserror::Error;
use uuid::Uuid;

/// Core module errors
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Store not found: {0}")]
    StoreNotFound(Uuid),

    #[error("Store is inactive: {0}")]
    StoreInactive(Uuid),

    #[error("Terminal not found: {0}")]
    TerminalNotFound(Uuid),

    #[error("Terminal is inactive: {0}")]
    TerminalInactive(Uuid),

    #[error("Terminal code already exists: {0}")]
    TerminalCodeExists(String),

    #[error("Invalid terminal code format")]
    InvalidTerminalCode,

    #[error("Invalid CAI number format")]
    InvalidCaiNumber,

    #[error("No CAI assigned to terminal: {0}")]
    NoCaiAssigned(Uuid),

    #[error("CAI has expired for terminal: {0}")]
    CaiExpired(Uuid),

    #[error("CAI range exhausted for terminal: {0}")]
    CaiRangeExhausted(Uuid),

    #[error("CAI range overlaps with existing active range")]
    CaiRangeOverlap,

    #[error("Invalid CAI range: start must be <= end")]
    InvalidCaiRange,

    #[error("Unauthorized: requires super_admin role")]
    Unauthorized,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// CAI-specific errors
#[derive(Debug)]
pub enum CaiError {
    Expired,
    RangeExhausted,
}
