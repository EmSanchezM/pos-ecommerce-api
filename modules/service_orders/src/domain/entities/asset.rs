//! Asset — a customer-owned thing being serviced (vehicle, equipment, room
//! appliance, electronic). Belongs to a store; optionally linked to a
//! `Customer` for known walk-ins. The "service history" of an asset is just
//! `SELECT * FROM service_orders WHERE asset_id = X` — no separate visit
//! table.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::domain::value_objects::{AssetId, AssetType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    id: AssetId,
    store_id: Uuid,
    customer_id: Option<Uuid>,
    asset_type: AssetType,
    brand: Option<String>,
    model: Option<String>,
    identifier: Option<String>,
    year: Option<i32>,
    color: Option<String>,
    description: Option<String>,
    attributes: JsonValue,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Asset {
    #[allow(clippy::too_many_arguments)]
    pub fn register(
        store_id: Uuid,
        customer_id: Option<Uuid>,
        asset_type: AssetType,
        brand: Option<String>,
        model: Option<String>,
        identifier: Option<String>,
        year: Option<i32>,
        color: Option<String>,
        description: Option<String>,
        attributes: JsonValue,
    ) -> Result<Self, ServiceOrdersError> {
        if let Some(y) = year
            && !(1900..=2100).contains(&y)
        {
            return Err(ServiceOrdersError::Validation(format!(
                "year {} is out of plausible range 1900..=2100",
                y
            )));
        }
        let now = Utc::now();
        Ok(Self {
            id: AssetId::new(),
            store_id,
            customer_id,
            asset_type,
            brand,
            model,
            identifier,
            year,
            color,
            description,
            attributes,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: AssetId,
        store_id: Uuid,
        customer_id: Option<Uuid>,
        asset_type: AssetType,
        brand: Option<String>,
        model: Option<String>,
        identifier: Option<String>,
        year: Option<i32>,
        color: Option<String>,
        description: Option<String>,
        attributes: JsonValue,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            customer_id,
            asset_type,
            brand,
            model,
            identifier,
            year,
            color,
            description,
            attributes,
            is_active,
            created_at,
            updated_at,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_details(
        &mut self,
        brand: Option<String>,
        model: Option<String>,
        identifier: Option<String>,
        year: Option<i32>,
        color: Option<String>,
        description: Option<String>,
        attributes: JsonValue,
    ) {
        self.brand = brand;
        self.model = model;
        self.identifier = identifier;
        self.year = year;
        self.color = color;
        self.description = description;
        self.attributes = attributes;
        self.updated_at = Utc::now();
    }

    pub fn link_customer(&mut self, customer_id: Uuid) {
        self.customer_id = Some(customer_id);
        self.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> AssetId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn customer_id(&self) -> Option<Uuid> {
        self.customer_id
    }
    pub fn asset_type(&self) -> AssetType {
        self.asset_type
    }
    pub fn brand(&self) -> Option<&str> {
        self.brand.as_deref()
    }
    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }
    pub fn year(&self) -> Option<i32> {
        self.year
    }
    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn attributes(&self) -> &JsonValue {
        &self.attributes
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
