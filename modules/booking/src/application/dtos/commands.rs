use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::ResourceType;

// -----------------------------------------------------------------------------
// Resources
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResourceCommand {
    pub store_id: Uuid,
    pub resource_type: ResourceType,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResourceCommand {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarWindowDto {
    pub day_of_week: i16,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetResourceCalendarCommand {
    pub windows: Vec<CalendarWindowDto>,
}

// -----------------------------------------------------------------------------
// Services
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceCommand {
    pub store_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub price: Decimal,
    pub buffer_minutes_before: Option<i32>,
    pub buffer_minutes_after: Option<i32>,
    pub requires_deposit: Option<bool>,
    pub deposit_amount: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceCommand {
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub price: Decimal,
    pub buffer_minutes_before: Option<i32>,
    pub buffer_minutes_after: Option<i32>,
    pub requires_deposit: Option<bool>,
    pub deposit_amount: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignServiceResourcesCommand {
    pub resource_ids: Vec<Uuid>,
}

// -----------------------------------------------------------------------------
// Appointments (auth admin)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAppointmentAdminCommand {
    pub store_id: Uuid,
    pub service_id: Uuid,
    pub resource_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelAppointmentCommand {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteAppointmentCommand {
    /// Optional sale id if the calling layer already created an invoice.
    pub generated_sale_id: Option<Uuid>,
}

// -----------------------------------------------------------------------------
// Public booking
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckAvailabilityQuery {
    pub service_id: Uuid,
    pub date: NaiveDate,
    /// Optional: restrict to a single resource.
    pub resource_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicBookCommand {
    pub service_id: Uuid,
    /// Optional — if None, the use case picks the first resource with the slot
    /// available at `starts_at`.
    pub resource_id: Option<Uuid>,
    pub starts_at: DateTime<Utc>,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: Option<String>,
    pub notes: Option<String>,
}

// -----------------------------------------------------------------------------
// Booking policy
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertBookingPolicyCommand {
    pub store_id: Uuid,
    pub requires_deposit: bool,
    pub deposit_percentage: Option<Decimal>,
    pub cancellation_window_hours: i32,
    pub no_show_fee_amount: Option<Decimal>,
    pub default_buffer_minutes: i32,
    pub advance_booking_days_max: i32,
}
