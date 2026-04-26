//! DeliveryProviderAdapter trait — every external courier implements this.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::ShippingError;

/// Input data for `dispatch()`. Includes everything the provider needs to
/// pick up the package and deliver it.
#[derive(Debug, Clone)]
pub struct DispatchRequest {
    pub shipment_idempotency_key: String,
    pub recipient_name: String,
    pub recipient_phone: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: Option<String>,
    pub country: String,
    pub weight_kg: Option<Decimal>,
    pub cash_to_collect: Option<Decimal>,
    pub notes: Option<String>,
}

/// What the provider returns after accepting the package.
#[derive(Debug, Clone)]
pub struct DispatchResult {
    pub provider_tracking_id: String,
    pub carrier_name: Option<String>,
    pub estimated_delivery: Option<DateTime<Utc>>,
    pub raw_response: Option<String>,
}

/// Decoded webhook event.
#[derive(Debug, Clone)]
pub struct ProviderWebhookEvent {
    pub event_type: String,
    pub provider_tracking_id: Option<String>,
    pub new_status: Option<String>,
    pub raw_payload: String,
}

#[async_trait]
pub trait DeliveryProviderAdapter: Send + Sync {
    async fn dispatch(&self, req: DispatchRequest) -> Result<DispatchResult, ShippingError>;

    async fn cancel(&self, provider_tracking_id: &str) -> Result<(), ShippingError>;

    async fn verify_webhook(
        &self,
        body: &str,
        signature: &str,
    ) -> Result<ProviderWebhookEvent, ShippingError>;

    /// True when the adapter is offline-only (Manual). The use case will
    /// fall back to caller-supplied tracking number/carrier instead of
    /// expecting a real `dispatch()` round-trip.
    fn is_manual(&self) -> bool {
        false
    }
}
