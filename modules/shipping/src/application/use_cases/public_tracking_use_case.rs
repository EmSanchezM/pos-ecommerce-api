//! Public tracking — exposed without authentication via tracking_number.

use std::sync::Arc;

use crate::ShippingError;
use crate::application::dtos::{PublicTrackingResponse, TrackingEventResponse};
use crate::domain::repositories::{ShipmentRepository, ShipmentTrackingEventRepository};

pub struct PublicTrackingUseCase {
    shipment_repo: Arc<dyn ShipmentRepository>,
    event_repo: Arc<dyn ShipmentTrackingEventRepository>,
}

impl PublicTrackingUseCase {
    pub fn new(
        shipment_repo: Arc<dyn ShipmentRepository>,
        event_repo: Arc<dyn ShipmentTrackingEventRepository>,
    ) -> Self {
        Self {
            shipment_repo,
            event_repo,
        }
    }

    pub async fn execute(
        &self,
        tracking_number: &str,
    ) -> Result<PublicTrackingResponse, ShippingError> {
        let shipment = self
            .shipment_repo
            .find_by_tracking(tracking_number)
            .await?
            .ok_or_else(|| ShippingError::ShipmentNotFound(uuid::Uuid::nil()))?;
        let events = self.event_repo.find_by_shipment(shipment.id()).await?;
        Ok(PublicTrackingResponse {
            tracking_number: shipment
                .tracking_number()
                .unwrap_or(tracking_number)
                .to_string(),
            status: shipment.status().to_string(),
            method_type: shipment.method_type().to_string(),
            carrier_name: shipment.carrier_name().map(str::to_string),
            city: shipment.city().to_string(),
            state: shipment.state().to_string(),
            country: shipment.country().to_string(),
            estimated_delivery: shipment.estimated_delivery(),
            shipped_at: shipment.shipped_at(),
            delivered_at: shipment.delivered_at(),
            events: events
                .into_iter()
                .map(TrackingEventResponse::from)
                .collect(),
        })
    }
}
