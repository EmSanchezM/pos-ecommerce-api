use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::Driver;

#[derive(Debug, Deserialize)]
pub struct CreateDriverCommand {
    pub store_id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub phone: String,
    pub vehicle_type: String,
    pub license_plate: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateDriverCommand {
    #[serde(default)]
    pub driver_id: Uuid,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub vehicle_type: Option<String>,
    pub license_plate: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub current_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DriverResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub phone: String,
    pub vehicle_type: String,
    pub license_plate: Option<String>,
    pub is_active: bool,
    pub current_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Driver> for DriverResponse {
    fn from(d: Driver) -> Self {
        Self {
            id: d.id().into_uuid(),
            store_id: d.store_id().into_uuid(),
            user_id: d.user_id().map(|u| u.into_uuid()),
            name: d.name().to_string(),
            phone: d.phone().to_string(),
            vehicle_type: d.vehicle_type().to_string(),
            license_plate: d.license_plate().map(str::to_string),
            is_active: d.is_active(),
            current_status: d.current_status().to_string(),
            created_at: d.created_at(),
            updated_at: d.updated_at(),
        }
    }
}
