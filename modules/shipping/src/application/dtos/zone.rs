use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::ShippingZone;

#[derive(Debug, Deserialize)]
pub struct CreateShippingZoneCommand {
    pub store_id: Uuid,
    pub name: String,
    pub countries: Vec<String>,
    pub states: Vec<String>,
    pub zip_codes: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateShippingZoneCommand {
    #[serde(default)]
    pub zone_id: Uuid,
    pub name: Option<String>,
    pub countries: Option<Vec<String>>,
    pub states: Option<Vec<String>>,
    pub zip_codes: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ShippingZoneResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub countries: Vec<String>,
    pub states: Vec<String>,
    pub zip_codes: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ShippingZone> for ShippingZoneResponse {
    fn from(z: ShippingZone) -> Self {
        Self {
            id: z.id().into_uuid(),
            store_id: z.store_id().into_uuid(),
            name: z.name().to_string(),
            countries: z.countries().to_vec(),
            states: z.states().to_vec(),
            zip_codes: z.zip_codes().to_vec(),
            is_active: z.is_active(),
            created_at: z.created_at(),
            updated_at: z.updated_at(),
        }
    }
}
