//! Uber Eats delivery adapter — STUB.

use async_trait::async_trait;

use super::provider_adapter::{
    DeliveryProviderAdapter, DispatchRequest, DispatchResult, ProviderWebhookEvent,
};
use crate::ShippingError;

#[derive(Debug, Default, Clone)]
pub struct UberEatsAdapter;

impl UberEatsAdapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str =
    "Uber Eats adapter is not yet wired. Configure credentials and implement the Uber Direct API.";

#[async_trait]
impl DeliveryProviderAdapter for UberEatsAdapter {
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
