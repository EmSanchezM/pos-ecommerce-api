//! PayPal gateway adapter — STUB.
//!
//! PayPal Honduras-friendly accounts are limited; full merchant accounts also
//! require an LLC. When wiring this adapter, use the PayPal REST API
//! (`/v2/checkout/orders`, `/v2/payments/captures/{id}/refund`) and verify
//! webhooks with `PAYPAL-TRANSMISSION-SIG` against `webhook_secret`.

use async_trait::async_trait;
use rust_decimal::Decimal;

use super::gateway_adapter::{
    GatewayAdapter, GatewayChargeResult, GatewayRefundResult, WebhookEvent,
};
use crate::PaymentsError;

#[derive(Debug, Default, Clone)]
pub struct PayPalAdapter;

impl PayPalAdapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str =
    "PayPal adapter is not yet wired. Configure credentials and implement the PayPal REST calls.";

#[async_trait]
impl GatewayAdapter for PayPalAdapter {
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
