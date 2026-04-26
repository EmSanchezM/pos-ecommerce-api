use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::ShipmentTrackingEvent;

#[derive(Debug, Serialize)]
pub struct TrackingEventResponse {
    pub id: Uuid,
    pub shipment_id: Uuid,
    pub status: String,
    pub notes: Option<String>,
    pub location_lat: Option<Decimal>,
    pub location_lng: Option<Decimal>,
    pub source: String,
    pub actor_user_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
}

impl From<ShipmentTrackingEvent> for TrackingEventResponse {
    fn from(e: ShipmentTrackingEvent) -> Self {
        Self {
            id: e.id().into_uuid(),
            shipment_id: e.shipment_id().into_uuid(),
            status: e.status().to_string(),
            notes: e.notes().map(str::to_string),
            location_lat: e.location_lat(),
            location_lng: e.location_lng(),
            source: e.source().to_string(),
            actor_user_id: e.actor_user_id().map(|u| u.into_uuid()),
            occurred_at: e.occurred_at(),
        }
    }
}
