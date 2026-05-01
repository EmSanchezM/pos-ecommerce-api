//! ShipmentTrackingEvent - immutable audit row for status / location changes.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{
    ShipmentId, ShipmentStatus, ShipmentTrackingEventId, TrackingEventSource,
};
use identity::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipmentTrackingEvent {
    id: ShipmentTrackingEventId,
    shipment_id: ShipmentId,
    status: ShipmentStatus,
    notes: Option<String>,
    location_lat: Option<Decimal>,
    location_lng: Option<Decimal>,
    source: TrackingEventSource,
    actor_user_id: Option<UserId>,
    raw_payload: Option<String>,
    occurred_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl ShipmentTrackingEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn record(
        shipment_id: ShipmentId,
        status: ShipmentStatus,
        source: TrackingEventSource,
        actor_user_id: Option<UserId>,
        notes: Option<String>,
        location_lat: Option<Decimal>,
        location_lng: Option<Decimal>,
        raw_payload: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ShipmentTrackingEventId::new(),
            shipment_id,
            status,
            notes,
            location_lat,
            location_lng,
            source,
            actor_user_id,
            raw_payload,
            occurred_at: now,
            created_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ShipmentTrackingEventId,
        shipment_id: ShipmentId,
        status: ShipmentStatus,
        notes: Option<String>,
        location_lat: Option<Decimal>,
        location_lng: Option<Decimal>,
        source: TrackingEventSource,
        actor_user_id: Option<UserId>,
        raw_payload: Option<String>,
        occurred_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            shipment_id,
            status,
            notes,
            location_lat,
            location_lng,
            source,
            actor_user_id,
            raw_payload,
            occurred_at,
            created_at,
        }
    }

    pub fn id(&self) -> ShipmentTrackingEventId {
        self.id
    }
    pub fn shipment_id(&self) -> ShipmentId {
        self.shipment_id
    }
    pub fn status(&self) -> ShipmentStatus {
        self.status
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn location_lat(&self) -> Option<Decimal> {
        self.location_lat
    }
    pub fn location_lng(&self) -> Option<Decimal> {
        self.location_lng
    }
    pub fn source(&self) -> TrackingEventSource {
        self.source
    }
    pub fn actor_user_id(&self) -> Option<UserId> {
        self.actor_user_id
    }
    pub fn raw_payload(&self) -> Option<&str> {
        self.raw_payload.as_deref()
    }
    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
