use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::BookingError;
use crate::domain::entities::Appointment;
use crate::domain::value_objects::{AppointmentId, AppointmentStatus, ResourceId};

#[derive(Debug, Clone, Default)]
pub struct ListAppointmentsFilters {
    pub store_id: Option<Uuid>,
    pub resource_id: Option<ResourceId>,
    pub customer_id: Option<Uuid>,
    pub status: Option<AppointmentStatus>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[async_trait]
pub trait AppointmentRepository: Send + Sync {
    /// Insert. Implementations must wrap in a transaction with a slot conflict
    /// check (`SELECT ... FOR UPDATE` over the resource's overlapping
    /// appointments) before inserting, returning `BookingError::SlotConflict`.
    async fn save_with_slot_check(&self, appointment: &Appointment) -> Result<(), BookingError>;

    async fn find_by_id(&self, id: AppointmentId) -> Result<Option<Appointment>, BookingError>;

    async fn find_by_public_token(&self, token: &str) -> Result<Option<Appointment>, BookingError>;

    /// Returns appointments occupying the slot grid for `resource_id` between
    /// `from` and `to`. Used by the availability use case to subtract booked
    /// time. Status filter is hard-coded to {Scheduled, Confirmed, InProgress}.
    async fn list_occupying_slots(
        &self,
        resource_id: ResourceId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Appointment>, BookingError>;

    async fn list(
        &self,
        filters: ListAppointmentsFilters,
    ) -> Result<Vec<Appointment>, BookingError>;

    /// Persist the full mutable surface of an existing appointment (status,
    /// generated_sale_id, deposit_transaction_id, canceled_reason, no_show_at,
    /// notes, updated_at). Reads `appointment.id` to target the row.
    async fn update(&self, appointment: &Appointment) -> Result<(), BookingError>;
}
