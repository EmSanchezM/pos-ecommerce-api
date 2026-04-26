//! PedidosYa delivery adapter — STUB.

use async_trait::async_trait;

use super::provider_adapter::{
    DeliveryProviderAdapter, DispatchRequest, DispatchResult, ProviderWebhookEvent,
};
use crate::ShippingError;

#[derive(Debug, Default, Clone)]
pub struct PedidosYaAdapter;

impl PedidosYaAdapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str =
    "PedidosYa adapter is not yet wired. Configure credentials and implement the PedidosYa API.";

#[async_trait]
impl DeliveryProviderAdapter for PedidosYaAdapter {
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
