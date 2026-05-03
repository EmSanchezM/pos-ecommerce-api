use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::value_objects::{RestaurantTableId, TableStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestaurantTable {
    id: RestaurantTableId,
    store_id: Uuid,
    label: String,
    capacity: i32,
    status: TableStatus,
    current_ticket_id: Option<Uuid>,
    notes: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl RestaurantTable {
    pub fn new(
        store_id: Uuid,
        label: String,
        capacity: i32,
        notes: Option<String>,
    ) -> Result<Self, RestaurantOperationsError> {
        if label.trim().is_empty() {
            return Err(RestaurantOperationsError::Validation(
                "label is required".to_string(),
            ));
        }
        if capacity <= 0 {
            return Err(RestaurantOperationsError::Validation(
                "capacity must be > 0".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: RestaurantTableId::new(),
            store_id,
            label,
            capacity,
            status: TableStatus::Free,
            current_ticket_id: None,
            notes,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: RestaurantTableId,
        store_id: Uuid,
        label: String,
        capacity: i32,
        status: TableStatus,
        current_ticket_id: Option<Uuid>,
        notes: Option<String>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            label,
            capacity,
            status,
            current_ticket_id,
            notes,
            is_active,
            created_at,
            updated_at,
        }
    }

    pub fn rename(&mut self, label: String, capacity: i32, notes: Option<String>) {
        self.label = label;
        self.capacity = capacity;
        self.notes = notes;
        self.updated_at = Utc::now();
    }

    pub fn set_status(&mut self, status: TableStatus, current_ticket_id: Option<Uuid>) {
        self.status = status;
        // When the table goes back to Free or Dirty, drop any lingering ticket
        // pointer; when it gets seated/reserved, the caller may attach one.
        self.current_ticket_id = current_ticket_id;
        self.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> RestaurantTableId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn capacity(&self) -> i32 {
        self.capacity
    }
    pub fn status(&self) -> TableStatus {
        self.status
    }
    pub fn current_ticket_id(&self) -> Option<Uuid> {
        self.current_ticket_id
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
