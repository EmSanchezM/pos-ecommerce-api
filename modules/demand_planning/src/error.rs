use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DemandPlanningError {
    #[error("Forecast not found: {0}")]
    ForecastNotFound(Uuid),

    #[error("Reorder policy not found: {0}")]
    ReorderPolicyNotFound(Uuid),

    #[error("Reorder policy already exists for variant {variant_id} at store {store_id}")]
    DuplicateReorderPolicy { variant_id: Uuid, store_id: Uuid },

    #[error("Replenishment suggestion not found: {0}")]
    SuggestionNotFound(Uuid),

    #[error("Cannot transition suggestion from {from} to {to}")]
    InvalidSuggestionTransition { from: String, to: String },

    #[error("Reorder policy invalid: max_qty must be >= min_qty")]
    InvalidPolicyRange,

    #[error("Reorder policy invalid: lead_time_days and review_cycle_days must be > 0")]
    InvalidPolicyDays,

    #[error("Negative quantity is not allowed")]
    NegativeQuantity,

    #[error("Insufficient history: {needed} points required, got {got}")]
    InsufficientHistory { needed: usize, got: usize },

    #[error("Optimistic lock conflict on reorder policy {0}")]
    PolicyVersionConflict(Uuid),

    #[error("Invalid forecast method: {0}")]
    InvalidForecastMethod(String),

    #[error("Invalid forecast period: {0}")]
    InvalidForecastPeriod(String),

    #[error("Invalid suggestion status: {0}")]
    InvalidSuggestionStatus(String),

    #[error("Invalid ABC class: {0}")]
    InvalidAbcClass(String),

    #[error("Forecasting failed: {0}")]
    ForecastingFailed(String),

    #[error("Dismiss reason is required")]
    DismissReasonRequired,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Subscriber error: {0}")]
    Subscriber(String),
}
