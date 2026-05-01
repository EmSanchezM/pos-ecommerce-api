//! Servientrega national courier adapter — STUB.

use async_trait::async_trait;

use super::provider_adapter::{
    DeliveryProviderAdapter, DispatchRequest, DispatchResult, ProviderWebhookEvent,
};
use crate::ShippingError;

#[derive(Debug, Default, Clone)]
pub struct ServientregaAdapter;

impl ServientregaAdapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str = "Servientrega adapter is not yet wired. Configure credentials and implement the Servientrega API.";

#[async_trait]
impl DeliveryProviderAdapter for ServientregaAdapter {
    async fn dispatch(&self, _req: DispatchRequest) -> Result<DispatchResult, ShippingError> {
        Err(ShippingError::ProviderError(NOT_WIRED.to_string()))
    }
    async fn cancel(&self, _provider_tracking_id: &str) -> Result<(), ShippingError> {
        Err(ShippingError::ProviderError(NOT_WIRED.to_string()))
    }
    async fn verify_webhook(
        &self,
        _body: &str,
        _signature: &str,
    ) -> Result<ProviderWebhookEvent, ShippingError> {
        Err(ShippingError::InvalidWebhookSignature)
    }
}
