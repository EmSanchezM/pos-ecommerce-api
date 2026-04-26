use async_trait::async_trait;
use sqlx::PgPool;

use crate::ShippingError;
use crate::domain::entities::ShipmentTrackingEvent;
use crate::domain::repositories::ShipmentTrackingEventRepository;
use crate::domain::value_objects::{
    ShipmentId, ShipmentStatus, ShipmentTrackingEventId, TrackingEventSource,
};
use identity::UserId;

pub struct PgShipmentTrackingEventRepository {
    pool: PgPool,
}

impl PgShipmentTrackingEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShipmentTrackingEventRepository for PgShipmentTrackingEventRepository {
    async fn save(&self, e: &ShipmentTrackingEvent) -> Result<(), ShippingError> {
        sqlx::query(
            r#"INSERT INTO shipment_tracking_events
              (id, shipment_id, status, notes, location_lat, location_lng,
               source, actor_user_id, raw_payload, occurred_at, created_at)
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#,
        )
        .bind(e.id().into_uuid())
        .bind(e.shipment_id().into_uuid())
        .bind(e.status().to_string())
        .bind(e.notes())
        .bind(e.location_lat())
        .bind(e.location_lng())
        .bind(e.source().to_string())
        .bind(e.actor_user_id().map(|u| u.into_uuid()))
        .bind(e.raw_payload())
        .bind(e.occurred_at())
        .bind(e.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_shipment(
        &self,
        shipment_id: ShipmentId,
    ) -> Result<Vec<ShipmentTrackingEvent>, ShippingError> {
        let rows = sqlx::query_as::<_, EventRow>(
            r#"SELECT id, shipment_id, status, notes, location_lat, location_lng,
                   source, actor_user_id, raw_payload, occurred_at, created_at
               FROM shipment_tracking_events
               WHERE shipment_id = $1
               ORDER BY occurred_at ASC"#,
        )
        .bind(shipment_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: uuid::Uuid,
    shipment_id: uuid::Uuid,
    status: String,
    notes: Option<String>,
    location_lat: Option<rust_decimal::Decimal>,
    location_lng: Option<rust_decimal::Decimal>,
    source: String,
    actor_user_id: Option<uuid::Uuid>,
    raw_payload: Option<String>,
    occurred_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<EventRow> for ShipmentTrackingEvent {
    type Error = ShippingError;
    fn try_from(row: EventRow) -> Result<Self, Self::Error> {
        let status: ShipmentStatus = row
            .status
            .parse()
            .map_err(|_| ShippingError::InvalidShipmentStatus)?;
        let source: TrackingEventSource = row
            .source
            .parse()
            .map_err(|_| ShippingError::InvalidTrackingSource)?;
        Ok(ShipmentTrackingEvent::reconstitute(
            ShipmentTrackingEventId::from_uuid(row.id),
            ShipmentId::from_uuid(row.shipment_id),
            status,
            row.notes,
            row.location_lat,
            row.location_lng,
            source,
            row.actor_user_id.map(UserId::from_uuid),
            row.raw_payload,
            row.occurred_at,
            row.created_at,
        ))
    }
}
