use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::value_objects::KitchenStationId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitchenStation {
    id: KitchenStationId,
    store_id: Uuid,
    name: String,
    color: Option<String>,
    sort_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl KitchenStation {
    pub fn new(
        store_id: Uuid,
        name: String,
        color: Option<String>,
        sort_order: i32,
    ) -> Result<Self, RestaurantOperationsError> {
        if name.trim().is_empty() {
            return Err(RestaurantOperationsError::Validation(
                "name is required".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: KitchenStationId::new(),
            store_id,
            name,
            color,
            sort_order,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: KitchenStationId,
        store_id: Uuid,
        name: String,
        color: Option<String>,
        sort_order: i32,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            name,
            color,
            sort_order,
            is_active,
            created_at,
            updated_at,
        }
    }

    pub fn rename(&mut self, name: String, color: Option<String>, sort_order: i32) {
        self.name = name;
        self.color = color;
        self.sort_order = sort_order;
        self.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> KitchenStationId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }
    pub fn sort_order(&self) -> i32 {
        self.sort_order
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
