//! HandleDeliveryWebhookUseCase — verify provider signature and translate
//! the event into a shipment status update.

use std::str::FromStr;
use std::sync::Arc;

use crate::ShippingError;
use crate::application::dtos::{DeliveryWebhookPayload, DeliveryWebhookResponse};
use crate::domain::entities::ShipmentTrackingEvent;
use crate::domain::repositories::{ShipmentRepository, ShipmentTrackingEventRepository};
use crate::domain::value_objects::{ShipmentStatus, TrackingEventSource};
use crate::infrastructure::adapters::DeliveryProviderRegistry;

pub struct HandleDeliveryWebhookUseCase {
    registry: Arc<dyn DeliveryProviderRegistry>,
    shipment_repo: Arc<dyn ShipmentRepository>,
    event_repo: Arc<dyn ShipmentTrackingEventRepository>,
}

impl HandleDeliveryWebhookUseCase {
    pub fn new(
        registry: Arc<dyn DeliveryProviderRegistry>,
        shipment_repo: Arc<dyn ShipmentRepository>,
        event_repo: Arc<dyn ShipmentTrackingEventRepository>,
    ) -> Self {
        Self {
            registry,
            shipment_repo,
            event_repo,
        }
    }

    pub async fn execute(
        &self,
        payload: DeliveryWebhookPayload,
    ) -> Result<DeliveryWebhookResponse, ShippingError> {
        let adapter = self.registry.for_type_str(&payload.provider_type)?;
        let event = adapter
            .verify_webhook(&payload.raw_body, &payload.signature)
            .await?;

        // Try to correlate with a shipment by provider tracking id.
        let mut shipment_id = None;
        if let Some(tid) = event.provider_tracking_id.as_deref()
            && let Some(mut shipment) = self.shipment_repo.find_by_tracking(tid).await?
        {
            shipment_id = Some(shipment.id().into_uuid());

            if let Some(new_status) = event.new_status.as_deref()
                && let Ok(status) = ShipmentStatus::from_str(new_status)
            {
                if shipment.transition_to(status).is_ok() {
                    self.shipment_repo.update(&shipment).await?;
                }
                let evt = ShipmentTrackingEvent::record(
                    shipment.id(),
                    status,
                    TrackingEventSource::Webhook,
                    None,
                    Some(format!("Webhook event: {}", event.event_type)),
                    None,
                    None,
                    Some(event.raw_payload.clone()),
                );
                self.event_repo.save(&evt).await?;
            }
        }

        Ok(DeliveryWebhookResponse {
            processed: true,
            shipment_id,
            event_type: event.event_type,
        })
    }
}
