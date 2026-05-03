use chrono::{DateTime, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{Appointment, BookingPolicy, Resource, ResourceCalendar, Service};
use crate::domain::value_objects::{AppointmentStatus, ResourceType, TimeSlot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub resource_type: ResourceType,
    pub name: String,
    pub color: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Resource> for ResourceResponse {
    fn from(r: &Resource) -> Self {
        Self {
            id: r.id().into_uuid(),
            store_id: r.store_id(),
            resource_type: r.resource_type(),
            name: r.name().to_string(),
            color: r.color().map(|s| s.to_string()),
            is_active: r.is_active(),
            created_at: r.created_at(),
            updated_at: r.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCalendarEntryResponse {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub day_of_week: i16,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_active: bool,
}

impl From<&ResourceCalendar> for ResourceCalendarEntryResponse {
    fn from(c: &ResourceCalendar) -> Self {
        Self {
            id: c.id().into_uuid(),
            resource_id: c.resource_id().into_uuid(),
            day_of_week: c.day_of_week(),
            start_time: c.start_time(),
            end_time: c.end_time(),
            is_active: c.is_active(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub price: Decimal,
    pub buffer_minutes_before: i32,
    pub buffer_minutes_after: i32,
    pub requires_deposit: bool,
    pub deposit_amount: Option<Decimal>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Service> for ServiceResponse {
    fn from(s: &Service) -> Self {
        Self {
            id: s.id().into_uuid(),
            store_id: s.store_id(),
            name: s.name().to_string(),
            description: s.description().map(|d| d.to_string()),
            duration_minutes: s.duration_minutes(),
            price: s.price(),
            buffer_minutes_before: s.buffer_minutes_before(),
            buffer_minutes_after: s.buffer_minutes_after(),
            requires_deposit: s.requires_deposit(),
            deposit_amount: s.deposit_amount(),
            is_active: s.is_active(),
            created_at: s.created_at(),
            updated_at: s.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub service_id: Uuid,
    pub resource_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub status: AppointmentStatus,
    pub deposit_transaction_id: Option<Uuid>,
    pub generated_sale_id: Option<Uuid>,
    pub notes: Option<String>,
    pub canceled_reason: Option<String>,
    pub no_show_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Appointment> for AppointmentResponse {
    fn from(a: &Appointment) -> Self {
        Self {
            id: a.id().into_uuid(),
            store_id: a.store_id(),
            service_id: a.service_id().into_uuid(),
            resource_id: a.resource_id().into_uuid(),
            customer_id: a.customer_id(),
            customer_name: a.customer_name().to_string(),
            customer_email: a.customer_email().to_string(),
            customer_phone: a.customer_phone().map(|s| s.to_string()),
            starts_at: a.starts_at(),
            ends_at: a.ends_at(),
            status: a.status(),
            deposit_transaction_id: a.deposit_transaction_id(),
            generated_sale_id: a.generated_sale_id(),
            notes: a.notes().map(|s| s.to_string()),
            canceled_reason: a.canceled_reason().map(|s| s.to_string()),
            no_show_at: a.no_show_at(),
            created_at: a.created_at(),
            updated_at: a.updated_at(),
        }
    }
}

/// Returned by `book_appointment_public` — includes the public_token so the
/// caller can build a "view your booking" URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicBookingResponse {
    #[serde(flatten)]
    pub appointment: AppointmentResponse,
    pub public_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAvailabilityResponse {
    pub resource_id: Uuid,
    pub resource_name: String,
    pub slots: Vec<TimeSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingPolicyResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub requires_deposit: bool,
    pub deposit_percentage: Option<Decimal>,
    pub cancellation_window_hours: i32,
    pub no_show_fee_amount: Option<Decimal>,
    pub default_buffer_minutes: i32,
    pub advance_booking_days_max: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&BookingPolicy> for BookingPolicyResponse {
    fn from(p: &BookingPolicy) -> Self {
        Self {
            id: p.id().into_uuid(),
            store_id: p.store_id(),
            requires_deposit: p.requires_deposit(),
            deposit_percentage: p.deposit_percentage(),
            cancellation_window_hours: p.cancellation_window_hours(),
            no_show_fee_amount: p.no_show_fee_amount(),
            default_buffer_minutes: p.default_buffer_minutes(),
            advance_booking_days_max: p.advance_booking_days_max(),
            created_at: p.created_at(),
            updated_at: p.updated_at(),
        }
    }
}
