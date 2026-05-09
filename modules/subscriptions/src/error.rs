use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error("Subscription plan not found: {0}")]
    PlanNotFound(Uuid),

    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(Uuid),

    #[error("Billing cycle not found: {0}")]
    BillingCycleNotFound(Uuid),

    #[error("Dunning attempt not found: {0}")]
    DunningAttemptNotFound(Uuid),

    #[error("Invalid status transition: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Organization {0} already has an active subscription")]
    OrganizationAlreadySubscribed(Uuid),

    #[error("Subscription plan is inactive: {0}")]
    PlanInactive(Uuid),

    #[error("Plan code already taken: {0}")]
    CodeAlreadyTaken(String),

    #[error("Invalid plan code: {0}")]
    InvalidPlanCode(String),

    #[error("Invalid subscription status string: {0}")]
    InvalidStatus(String),

    #[error("Invalid billing cycle status string: {0}")]
    InvalidCycleStatus(String),

    #[error("Invalid billing interval string: {0}")]
    InvalidInterval(String),

    #[error("Invalid dunning outcome string: {0}")]
    InvalidDunningOutcome(String),

    #[error("Subscription was modified concurrently — refresh and retry")]
    OptimisticLockFailed,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Fiscal integration failed: {0}")]
    FiscalIntegration(String),

    #[error("Payment integration failed: {0}")]
    PaymentIntegration(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
