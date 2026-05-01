use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::ShippingMethod;

#[derive(Debug, Deserialize)]
pub struct CreateShippingMethodCommand {
    pub store_id: Uuid,
    pub name: String,
    pub code: String,
    pub method_type: String,
    pub description: Option<String>,
    pub estimated_days_min: Option<i32>,
    pub estimated_days_max: Option<i32>,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateShippingMethodCommand {
    #[serde(default)]
    pub method_id: Uuid,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub estimated_days_min: Option<Option<i32>>,
    pub estimated_days_max: Option<Option<i32>>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ShippingMethodResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub code: String,
    pub method_type: String,
    pub description: Option<String>,
    pub estimated_days_min: Option<i32>,
    pub estimated_days_max: Option<i32>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ShippingMethod> for ShippingMethodResponse {
    fn from(m: ShippingMethod) -> Self {
        Self {
            id: m.id().into_uuid(),
            store_id: m.store_id().into_uuid(),
            name: m.name().to_string(),
            code: m.code().to_string(),
            method_type: m.method_type().to_string(),
            description: m.description().map(str::to_string),
            estimated_days_min: m.estimated_days_min(),
            estimated_days_max: m.estimated_days_max(),
            is_active: m.is_active(),
            sort_order: m.sort_order(),
            created_at: m.created_at(),
            updated_at: m.updated_at(),
        }
    }
}
