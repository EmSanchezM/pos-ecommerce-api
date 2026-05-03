use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RestaurantOperationsError {
    #[error("Kitchen station not found: {0}")]
    StationNotFound(Uuid),

    #[error("Restaurant table not found: {0}")]
    TableNotFound(Uuid),

    #[error("Modifier group not found: {0}")]
    ModifierGroupNotFound(Uuid),

    #[error("Modifier not found: {0}")]
    ModifierNotFound(Uuid),

    #[error("Product not found: {0}")]
    ProductNotFound(Uuid),

    #[error("KDS ticket not found: {0}")]
    TicketNotFound(Uuid),

    #[error("KDS ticket item not found: {0}")]
    ItemNotFound(Uuid),

    #[error("Invalid state transition for KDS ticket: {from} → {to}")]
    InvalidTicketStateTransition { from: String, to: String },

    #[error("Invalid state transition for KDS item: {from} → {to}")]
    InvalidItemStateTransition { from: String, to: String },

    #[error("Cannot modify a terminal ticket (served or canceled)")]
    CannotModifyTerminalTicket,

    #[error("Invalid kitchen station status: {0}")]
    InvalidStationStatus(String),

    #[error("Invalid table status: {0}")]
    InvalidTableStatus(String),

    #[error("Invalid table status transition: {from} → {to}")]
    InvalidTableStatusTransition { from: String, to: String },

    #[error("Invalid KDS ticket status: {0}")]
    InvalidTicketStatus(String),

    #[error("Invalid KDS item status: {0}")]
    InvalidItemStatus(String),

    #[error("Invalid course: {0}")]
    InvalidCourse(String),

    #[error("Modifier group {group_id} requires between {min} and {max} selections (got {got})")]
    ModifierSelectionOutOfBounds {
        group_id: Uuid,
        min: i32,
        max: i32,
        got: i32,
    },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Subscriber error: {0}")]
    Subscriber(String),
}
