use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ServiceOrdersError {
    #[error("Asset not found: {0}")]
    AssetNotFound(Uuid),

    #[error("Service order not found: {0}")]
    ServiceOrderNotFound(Uuid),

    #[error("Service order item not found: {0}")]
    ItemNotFound(Uuid),

    #[error("Diagnostic not found: {0}")]
    DiagnosticNotFound(Uuid),

    #[error("Quote not found: {0}")]
    QuoteNotFound(Uuid),

    #[error("Customer not found: {0}")]
    CustomerNotFound(Uuid),

    #[error("Invalid state transition for service order: {from} → {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Invalid quote state transition: {from} → {to}")]
    InvalidQuoteStateTransition { from: String, to: String },

    #[error("Quote {0} is no longer the latest active quote for the order")]
    QuoteSuperseded(Uuid),

    #[error("Cannot modify a delivered or canceled service order")]
    CannotModifyTerminalOrder,

    #[error("Asset is inactive: {0}")]
    InactiveAsset(Uuid),

    #[error("Invalid asset type: {0}")]
    InvalidAssetType(String),

    #[error("Invalid service order status: {0}")]
    InvalidServiceOrderStatus(String),

    #[error("Invalid service order priority: {0}")]
    InvalidServiceOrderPriority(String),

    #[error("Invalid item type: {0}")]
    InvalidItemType(String),

    #[error("Invalid quote status: {0}")]
    InvalidQuoteStatus(String),

    #[error("Invalid diagnostic severity: {0}")]
    InvalidDiagnosticSeverity(String),

    #[error("Invalid public token")]
    InvalidPublicToken,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Subscriber error: {0}")]
    Subscriber(String),
}
