//! KdsTicket — kitchen display ticket aggregate root.
//!
//! ```text
//! Pending ──send──▶ InProgress ──ready──▶ Ready ──serve──▶ Served
//!    │                  │                    │
//!    └──cancel──▶ Canceled (desde Pending o InProgress)
//! ```
//!
//! Auto-transitions are driven by item status changes from the application
//! layer (the entity exposes the per-status setters; the use case checks
//! "all items advanced" and calls the matching transition here).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::value_objects::{
    Course, KdsTicketId, KdsTicketStatus, KitchenStationId, RestaurantTableId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdsTicket {
    id: KdsTicketId,
    store_id: Uuid,
    station_id: KitchenStationId,
    table_id: Option<RestaurantTableId>,
    sale_id: Option<Uuid>,
    ticket_number: i32,
    status: KdsTicketStatus,
    course: Course,
    notes: Option<String>,
    sent_at: Option<DateTime<Utc>>,
    ready_at: Option<DateTime<Utc>>,
    served_at: Option<DateTime<Utc>>,
    canceled_reason: Option<String>,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl KdsTicket {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store_id: Uuid,
        station_id: KitchenStationId,
        table_id: Option<RestaurantTableId>,
        sale_id: Option<Uuid>,
        ticket_number: i32,
        course: Course,
        notes: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<Self, RestaurantOperationsError> {
        if ticket_number <= 0 {
            return Err(RestaurantOperationsError::Validation(
                "ticket_number must be > 0".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: KdsTicketId::new(),
            store_id,
            station_id,
            table_id,
            sale_id,
            ticket_number,
            status: KdsTicketStatus::Pending,
            course,
            notes,
            sent_at: None,
            ready_at: None,
            served_at: None,
            canceled_reason: None,
            created_by,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: KdsTicketId,
        store_id: Uuid,
        station_id: KitchenStationId,
        table_id: Option<RestaurantTableId>,
        sale_id: Option<Uuid>,
        ticket_number: i32,
        status: KdsTicketStatus,
        course: Course,
        notes: Option<String>,
        sent_at: Option<DateTime<Utc>>,
        ready_at: Option<DateTime<Utc>>,
        served_at: Option<DateTime<Utc>>,
        canceled_reason: Option<String>,
        created_by: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            station_id,
            table_id,
            sale_id,
            ticket_number,
            status,
            course,
            notes,
            sent_at,
            ready_at,
            served_at,
            canceled_reason,
            created_by,
            created_at,
            updated_at,
        }
    }

    fn transition(&mut self, to: KdsTicketStatus) -> Result<(), RestaurantOperationsError> {
        if !self.status.can_transition_to(to) {
            return Err(RestaurantOperationsError::InvalidTicketStateTransition {
                from: self.status.as_str().to_string(),
                to: to.as_str().to_string(),
            });
        }
        self.status = to;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn send(&mut self) -> Result<(), RestaurantOperationsError> {
        self.transition(KdsTicketStatus::InProgress)?;
        self.sent_at = Some(self.updated_at);
        Ok(())
    }

    pub fn mark_ready(&mut self) -> Result<(), RestaurantOperationsError> {
        self.transition(KdsTicketStatus::Ready)?;
        self.ready_at = Some(self.updated_at);
        Ok(())
    }

    pub fn serve(&mut self) -> Result<(), RestaurantOperationsError> {
        self.transition(KdsTicketStatus::Served)?;
        self.served_at = Some(self.updated_at);
        Ok(())
    }

    pub fn cancel(&mut self, reason: String) -> Result<(), RestaurantOperationsError> {
        if reason.trim().is_empty() {
            return Err(RestaurantOperationsError::Validation(
                "cancel reason is required".to_string(),
            ));
        }
        self.transition(KdsTicketStatus::Canceled)?;
        self.canceled_reason = Some(reason);
        Ok(())
    }

    pub fn id(&self) -> KdsTicketId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn station_id(&self) -> KitchenStationId {
        self.station_id
    }
    pub fn table_id(&self) -> Option<RestaurantTableId> {
        self.table_id
    }
    pub fn sale_id(&self) -> Option<Uuid> {
        self.sale_id
    }
    pub fn ticket_number(&self) -> i32 {
        self.ticket_number
    }
    pub fn status(&self) -> KdsTicketStatus {
        self.status
    }
    pub fn course(&self) -> Course {
        self.course
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn sent_at(&self) -> Option<DateTime<Utc>> {
        self.sent_at
    }
    pub fn ready_at(&self) -> Option<DateTime<Utc>> {
        self.ready_at
    }
    pub fn served_at(&self) -> Option<DateTime<Utc>> {
        self.served_at
    }
    pub fn canceled_reason(&self) -> Option<&str> {
        self.canceled_reason.as_deref()
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
