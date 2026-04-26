//! BAC Credomatic gateway adapter — STUB.
//!
//! BAC offers a card-not-present gateway in Honduras through their
//! "BAC e-Pago" / Credomatic platform. Onboarding is direct (no LLC needed)
//! but contracts and SLAs are negotiated per merchant. Implementation will
//! talk to the merchant API documented in the contract pack.

use async_trait::async_trait;
use rust_decimal::Decimal;

use super::gateway_adapter::{
    GatewayAdapter, GatewayChargeResult, GatewayRefundResult, WebhookEvent,
};
use crate::PaymentsError;

#[derive(Debug, Default, Clone)]
pub struct BacCredomaticAdapter;

impl BacCredomaticAdapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str = "BAC Credomatic adapter is not yet wired. Configure credentials and implement the BAC e-Pago API calls.";

#[async_trait]
impl GatewayAdapter for BacCredomaticAdapter {
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
