//! Ficohsa gateway adapter — STUB.
//!
//! Ficohsa offers a merchant gateway for card payments in Honduras
//! (CardPay / Ficohsa Pay). Like BAC, onboarding is local but the API
//! is contract-specific.

use async_trait::async_trait;
use rust_decimal::Decimal;

use super::gateway_adapter::{
    GatewayAdapter, GatewayChargeResult, GatewayRefundResult, WebhookEvent,
};
use crate::PaymentsError;

#[derive(Debug, Default, Clone)]
pub struct FicohsaAdapter;

impl FicohsaAdapter {
    pub fn new() -> Self {
        Self
    }
}

const NOT_WIRED: &str = "Ficohsa adapter is not yet wired. Configure credentials and implement the Ficohsa Pay API calls.";

#[async_trait]
impl GatewayAdapter for FicohsaAdapter {
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
