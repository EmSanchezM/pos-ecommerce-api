//! ShippingMethod - catalog row (per store) describing one way to ship.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{ShippingMethodId, ShippingMethodType};
use identity::StoreId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingMethod {
    id: ShippingMethodId,
    store_id: StoreId,
    name: String,
    code: String,
    method_type: ShippingMethodType,
    description: Option<String>,
    estimated_days_min: Option<i32>,
    estimated_days_max: Option<i32>,
    is_active: bool,
    sort_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ShippingMethod {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        store_id: StoreId,
        name: String,
        code: String,
        method_type: ShippingMethodType,
        description: Option<String>,
        estimated_days_min: Option<i32>,
        estimated_days_max: Option<i32>,
        sort_order: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ShippingMethodId::new(),
            store_id,
            name,
            code,
            method_type,
            description,
            estimated_days_min,
            estimated_days_max,
            is_active: true,
            sort_order,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ShippingMethodId,
        store_id: StoreId,
        name: String,
        code: String,
        method_type: ShippingMethodType,
        description: Option<String>,
        estimated_days_min: Option<i32>,
        estimated_days_max: Option<i32>,
        is_active: bool,
        sort_order: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            name,
            code,
            method_type,
            description,
            estimated_days_min,
            estimated_days_max,
            is_active,
            sort_order,
            created_at,
            updated_at,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.touch();
    }
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.touch();
    }
    pub fn set_estimated_days(&mut self, min: Option<i32>, max: Option<i32>) {
        self.estimated_days_min = min;
        self.estimated_days_max = max;
        self.touch();
    }
    pub fn set_sort_order(&mut self, sort_order: i32) {
        self.sort_order = sort_order;
        self.touch();
    }
    pub fn activate(&mut self) {
        self.is_active = true;
        self.touch();
    }
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.touch();
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> ShippingMethodId {
        self.id
    }
    pub fn store_id(&self) -> StoreId {
        self.store_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn method_type(&self) -> ShippingMethodType {
        self.method_type
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn estimated_days_min(&self) -> Option<i32> {
        self.estimated_days_min
    }
    pub fn estimated_days_max(&self) -> Option<i32> {
        self.estimated_days_max
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
