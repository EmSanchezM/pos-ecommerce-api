use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::value_objects::MenuModifierGroupId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuModifierGroup {
    id: MenuModifierGroupId,
    store_id: Uuid,
    name: String,
    min_select: i32,
    max_select: i32,
    sort_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl MenuModifierGroup {
    pub fn new(
        store_id: Uuid,
        name: String,
        min_select: i32,
        max_select: i32,
        sort_order: i32,
    ) -> Result<Self, RestaurantOperationsError> {
        if name.trim().is_empty() {
            return Err(RestaurantOperationsError::Validation(
                "name is required".to_string(),
            ));
        }
        if min_select < 0 || max_select < min_select {
            return Err(RestaurantOperationsError::Validation(format!(
                "invalid select bounds: min={} max={}",
                min_select, max_select
            )));
        }
        let now = Utc::now();
        Ok(Self {
            id: MenuModifierGroupId::new(),
            store_id,
            name,
            min_select,
            max_select,
            sort_order,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: MenuModifierGroupId,
        store_id: Uuid,
        name: String,
        min_select: i32,
        max_select: i32,
        sort_order: i32,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            name,
            min_select,
            max_select,
            sort_order,
            is_active,
            created_at,
            updated_at,
        }
    }

    pub fn update(
        &mut self,
        name: String,
        min_select: i32,
        max_select: i32,
        sort_order: i32,
    ) -> Result<(), RestaurantOperationsError> {
        if name.trim().is_empty() {
            return Err(RestaurantOperationsError::Validation(
                "name is required".to_string(),
            ));
        }
        if min_select < 0 || max_select < min_select {
            return Err(RestaurantOperationsError::Validation(format!(
                "invalid select bounds: min={} max={}",
                min_select, max_select
            )));
        }
        self.name = name;
        self.min_select = min_select;
        self.max_select = max_select;
        self.sort_order = sort_order;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn id(&self) -> MenuModifierGroupId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn min_select(&self) -> i32 {
        self.min_select
    }
    pub fn max_select(&self) -> i32 {
        self.max_select
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
