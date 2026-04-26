//! Stripe gateway adapter — STUB.
//!
//! Stripe requires an international LLC (US/EU) to onboard, so this adapter
//! is left as a `not yet implemented` stub. When the LLC is in place the
//! `charge`, `refund` and `verify_webhook` methods need to be filled in
//! using the `stripe-rust` crate (or direct REST calls via `reqwest`) and
//! the per-gateway `GatewayConfig` (api_key/secret_key) loaded from the
//! `payment_gateways` row.

use async_trait::async_trait;
use rust_decimal::Decimal;

use super::gateway_adapter::{
    GatewayAdapter, GatewayChargeResult, GatewayRefundResult, WebhookEvent,
};
use crate::PaymentsError;

#[derive(Debug, Default, Clone)]
pub struct StripeAdapter;

impl StripeAdapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str =
    "Stripe adapter is not yet wired. Configure credentials and implement the Stripe API calls.";

#[async_trait]
impl GatewayAdapter for StripeAdapter {
    async fn charge(
        &self,
        _amount: Decimal,
        _currency: &str,
        _token: &str,
        _idempotency_key: &str,
    ) -> Result<GatewayChargeResult, PaymentsError> {
        Err(PaymentsError::ProcessingFailed(NOT_WIRED.to_string()))
    }

    async fn refund(
        &self,
        _gateway_tx_id: &str,
        _amount: Option<Decimal>,
        _reason: &str,
    ) -> Result<GatewayRefundResult, PaymentsError> {
        Err(PaymentsError::ProcessingFailed(NOT_WIRED.to_string()))
    }

    async fn verify_webhook(
        &self,
        _body: &str,
        _signature: &str,
    ) -> Result<WebhookEvent, PaymentsError> {
        Err(PaymentsError::InvalidWebhookSignature)
    }
}
