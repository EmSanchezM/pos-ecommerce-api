//! Appointment — the booking aggregate root. Owns the workflow:
//!
//! ```text
//! Scheduled ──confirm──▶ Confirmed ──start──▶ InProgress ──complete──▶ Completed
//!     │                       │                    │
//!     └──cancel──▶ Canceled ◀─┴──cancel──┘         └──no_show──▶ NoShow
//! ```
//!
//! `customer_id` is optional so public/unauthenticated bookings can capture a
//! contact via the snapshot fields without forcing a registered Customer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

use crate::BookingError;
use crate::domain::value_objects::{AppointmentId, AppointmentStatus, ResourceId, ServiceId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    id: AppointmentId,
    store_id: Uuid,
    service_id: ServiceId,
    resource_id: ResourceId,
    customer_id: Option<Uuid>,
    customer_name: String,
    customer_email: String,
    customer_phone: Option<String>,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    status: AppointmentStatus,
    deposit_transaction_id: Option<Uuid>,
    generated_sale_id: Option<Uuid>,
    notes: Option<String>,
    canceled_reason: Option<String>,
    no_show_at: Option<DateTime<Utc>>,
    public_token: String,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Appointment {
    #[allow(clippy::too_many_arguments)]
    pub fn schedule(
        store_id: Uuid,
        service_id: ServiceId,
        resource_id: ResourceId,
        customer_id: Option<Uuid>,
        customer_name: String,
        customer_email: String,
        customer_phone: Option<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        notes: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<Self, BookingError> {
        if ends_at <= starts_at {
            return Err(BookingError::InvalidTimeRange);
        }
        if customer_name.trim().is_empty() || customer_email.trim().is_empty() {
            return Err(BookingError::Validation(
                "customer_name and customer_email are required".to_string(),
            ));
        }
        let now = Utc::now();
        // Random URL-safe token for public lookup. UUID v7 is time-ordered but
        // its randomness is sufficient for an unguessable per-appointment token
        // (short URLs, no PII embedded).
        let public_token = Uuid::new_v7(Timestamp::now(NoContext)).simple().to_string();
        Ok(Self {
            id: AppointmentId::new(),
            store_id,
            service_id,
            resource_id,
            customer_id,
            customer_name,
            customer_email,
            customer_phone,
            starts_at,
            ends_at,
            status: AppointmentStatus::Scheduled,
            deposit_transaction_id: None,
            generated_sale_id: None,
            notes,
            canceled_reason: None,
            no_show_at: None,
            public_token,
            created_by,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: AppointmentId,
        store_id: Uuid,
        service_id: ServiceId,
        resource_id: ResourceId,
        customer_id: Option<Uuid>,
        customer_name: String,
        customer_email: String,
        customer_phone: Option<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        status: AppointmentStatus,
        deposit_transaction_id: Option<Uuid>,
        generated_sale_id: Option<Uuid>,
        notes: Option<String>,
        canceled_reason: Option<String>,
        no_show_at: Option<DateTime<Utc>>,
        public_token: String,
        created_by: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            service_id,
            resource_id,
            customer_id,
            customer_name,
            customer_email,
            customer_phone,
            starts_at,
            ends_at,
            status,
            deposit_transaction_id,
            generated_sale_id,
            notes,
            canceled_reason,
            no_show_at,
            public_token,
            created_by,
            created_at,
            updated_at,
        }
    }

    fn transition(&mut self, to: AppointmentStatus) -> Result<(), BookingError> {
        if !self.status.can_transition_to(to) {
            return Err(BookingError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: to.as_str().to_string(),
            });
        }
        self.status = to;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn confirm(&mut self) -> Result<(), BookingError> {
        self.transition(AppointmentStatus::Confirmed)
    }

    pub fn start(&mut self) -> Result<(), BookingError> {
        self.transition(AppointmentStatus::InProgress)
    }

    pub fn complete(&mut self, generated_sale_id: Option<Uuid>) -> Result<(), BookingError> {
        self.transition(AppointmentStatus::Completed)?;
        self.generated_sale_id = generated_sale_id;
        Ok(())
    }

    pub fn cancel(&mut self, reason: String) -> Result<(), BookingError> {
        self.transition(AppointmentStatus::Canceled)?;
        self.canceled_reason = Some(reason);
        Ok(())
    }

    pub fn mark_no_show(&mut self) -> Result<(), BookingError> {
        self.transition(AppointmentStatus::NoShow)?;
        self.no_show_at = Some(Utc::now());
        Ok(())
    }

    pub fn attach_deposit_transaction(&mut self, transaction_id: Uuid) {
        self.deposit_transaction_id = Some(transaction_id);
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> AppointmentId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn service_id(&self) -> ServiceId {
        self.service_id
    }
    pub fn resource_id(&self) -> ResourceId {
        self.resource_id
    }
    pub fn customer_id(&self) -> Option<Uuid> {
        self.customer_id
    }
    pub fn customer_name(&self) -> &str {
        &self.customer_name
    }
    pub fn customer_email(&self) -> &str {
        &self.customer_email
    }
    pub fn customer_phone(&self) -> Option<&str> {
        self.customer_phone.as_deref()
    }
    pub fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }
    pub fn ends_at(&self) -> DateTime<Utc> {
        self.ends_at
    }
    pub fn status(&self) -> AppointmentStatus {
        self.status
    }
    pub fn deposit_transaction_id(&self) -> Option<Uuid> {
        self.deposit_transaction_id
    }
    pub fn generated_sale_id(&self) -> Option<Uuid> {
        self.generated_sale_id
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn canceled_reason(&self) -> Option<&str> {
        self.canceled_reason.as_deref()
    }
    pub fn no_show_at(&self) -> Option<DateTime<Utc>> {
        self.no_show_at
    }
    pub fn public_token(&self) -> &str {
        &self.public_token
    }
    pub fn created_by(&self) -> Option<Uuid> {
        self.created_by
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
