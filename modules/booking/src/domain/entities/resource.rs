use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{ResourceId, ResourceType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    id: ResourceId,
    store_id: Uuid,
    resource_type: ResourceType,
    name: String,
    color: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Resource {
    pub fn new(
        store_id: Uuid,
        resource_type: ResourceType,
        name: String,
        color: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ResourceId::new(),
            store_id,
            resource_type,
            name,
            color,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ResourceId,
        store_id: Uuid,
        resource_type: ResourceType,
        name: String,
        color: Option<String>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            resource_type,
            name,
            color,
            is_active,
            created_at,
            updated_at,
        }
    }

    pub fn rename(&mut self, name: String, color: Option<String>) {
        self.name = name;
        self.color = color;
        self.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> ResourceId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
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
