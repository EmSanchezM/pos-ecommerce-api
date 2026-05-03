use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BookingError {
    #[error("Resource not found: {0}")]
    ResourceNotFound(Uuid),

    #[error("Service not found: {0}")]
    ServiceNotFound(Uuid),

    #[error("Appointment not found: {0}")]
    AppointmentNotFound(Uuid),

    #[error("Booking policy not found for store: {0}")]
    PolicyNotFound(Uuid),

    #[error("Customer not found: {0}")]
    CustomerNotFound(Uuid),

    #[error("Store not found: {0}")]
    StoreNotFound(Uuid),

    #[error("Resource {resource_id} is not eligible to perform service {service_id}")]
    ResourceNotEligibleForService { service_id: Uuid, resource_id: Uuid },

    #[error("Time slot {start} → {end} conflicts with an existing appointment")]
    SlotConflict {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    },

    #[error("Requested slot is outside the resource's working hours")]
    OutsideWorkingHours,

    #[error("Invalid state transition for appointment: {from} → {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Cancellation rejected: appointment starts in less than {window_hours}h")]
    OutsideCancellationWindow { window_hours: i32 },

    #[error("Invalid time range (end must be strictly after start)")]
    InvalidTimeRange,

    #[error("Invalid duration: {0} minutes")]
    InvalidDuration(i32),

    #[error("Invalid resource type: {0}")]
    InvalidResourceType(String),

    #[error("Invalid appointment status: {0}")]
    InvalidAppointmentStatus(String),

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
