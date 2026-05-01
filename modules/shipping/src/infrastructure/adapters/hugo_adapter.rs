//! Hugo (Honduras) delivery adapter — STUB.
//!
//! Hugo offers REST APIs for B2B partners. When implemented, this adapter
//! will speak to `https://api.hugoapp.com/...` using the per-provider
//! `api_key`/`secret_key` from the encrypted config.

use async_trait::async_trait;

use super::provider_adapter::{
    DeliveryProviderAdapter, DispatchRequest, DispatchResult, ProviderWebhookEvent,
};
use crate::ShippingError;

#[derive(Debug, Default, Clone)]
pub struct HugoAdapter;

impl HugoAdapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str =
    "Hugo adapter is not yet wired. Configure credentials and implement the Hugo API calls.";

#[async_trait]
impl DeliveryProviderAdapter for HugoAdapter {
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
