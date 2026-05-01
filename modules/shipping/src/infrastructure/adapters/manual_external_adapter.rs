//! Manual external adapter — for offline coordination.
//!
//! The most common HN scenario: the shop calls Hugo / Servientrega by
//! WhatsApp, the courier comes by, gives a tracking sticker by hand, and the
//! manager pastes the code into the system. This adapter never fails — it
//! just reflects whatever the user provides.

use async_trait::async_trait;
use chrono::Utc;

use super::provider_adapter::{
    DeliveryProviderAdapter, DispatchRequest, DispatchResult, ProviderWebhookEvent,
};
use crate::ShippingError;

#[derive(Debug, Default, Clone)]
pub struct ManualExternalAdapter;

impl ManualExternalAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DeliveryProviderAdapter for ManualExternalAdapter {
    async fn dispatch(&self, req: DispatchRequest) -> Result<DispatchResult, ShippingError> {
        // Generate a deterministic placeholder so the row exists; the real
        // tracking number is supplied by the caller via the dispatch use case.
        Ok(DispatchResult {
            provider_tracking_id: format!("manual_{}", req.shipment_idempotency_key),
            carrier_name: None,
            estimated_delivery: None,
            raw_response: None,
        })
    }

    async fn cancel(&self, _provider_tracking_id: &str) -> Result<(), ShippingError> {
        // No-op: the human cancels offline.
        Ok(())
    }

    async fn verify_webhook(
        &self,
        body: &str,
        _signature: &str,
    ) -> Result<ProviderWebhookEvent, ShippingError> {
        Ok(ProviderWebhookEvent {
            event_type: "manual.event".to_string(),
            provider_tracking_id: None,
            new_status: None,
            raw_payload: format!("[{}] {}", Utc::now().to_rfc3339(), body),
        })
    }

    fn is_manual(&self) -> bool {
        true
    }
}
