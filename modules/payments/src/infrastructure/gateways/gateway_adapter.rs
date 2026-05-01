//! GatewayAdapter trait — the abstraction every external provider implements.

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::PaymentsError;

/// Result returned by a gateway after authorising a charge.
#[derive(Debug, Clone)]
pub struct GatewayChargeResult {
    pub gateway_transaction_id: String,
    pub authorization_code: Option<String>,
    pub card_last_four: Option<String>,
    pub card_brand: Option<String>,
    pub raw_response: Option<String>,
}

/// Result returned by a gateway after a refund call.
#[derive(Debug, Clone)]
pub struct GatewayRefundResult {
    pub gateway_transaction_id: String,
    pub raw_response: Option<String>,
}

/// Decoded webhook event payload.
#[derive(Debug, Clone)]
pub struct WebhookEvent {
    pub event_type: String,
    pub gateway_transaction_id: Option<String>,
    pub raw_payload: String,
}

#[async_trait]
pub trait GatewayAdapter: Send + Sync {
    async fn charge(
        &self,
        amount: Decimal,
        currency: &str,
        token: &str,
        idempotency_key: &str,
    ) -> Result<GatewayChargeResult, PaymentsError>;

    async fn refund(
        &self,
        gateway_tx_id: &str,
        amount: Option<Decimal>,
        reason: &str,
    ) -> Result<GatewayRefundResult, PaymentsError>;

    async fn verify_webhook(
        &self,
        body: &str,
        signature: &str,
    ) -> Result<WebhookEvent, PaymentsError>;

    /// True when the adapter cannot autoconfirm a charge — the transaction
    /// stays `Pending` after `charge()` and a human must call `confirm()`
    /// (or the reconciliation use case must match it). Defaults to `false`
    /// for remote gateways like Stripe/PayPal.
    fn requires_manual_confirmation(&self) -> bool {
        false
    }
}
