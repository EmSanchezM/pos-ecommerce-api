//! Manual gateway adapter - used when payments are reconciled offline.
//!
//! For Honduras, this is the most common case: the customer pays via
//! `transferencia bancaria`, `depósito en agencia`, or contra-entrega
//! (cash on delivery). The adapter does not contact any external service —
//! it produces a deterministic transaction id from the idempotency key
//! and reports `requires_manual_confirmation = true`, so the use case
//! leaves the transaction in `Pending` until a manager confirms it (or
//! the reconciliation use case matches it against a bank statement).

use async_trait::async_trait;
use rust_decimal::Decimal;

use super::gateway_adapter::{
    GatewayAdapter, GatewayChargeResult, GatewayRefundResult, WebhookEvent,
};
use crate::PaymentsError;

#[derive(Debug, Default, Clone)]
pub struct ManualGatewayAdapter;

impl ManualGatewayAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GatewayAdapter for ManualGatewayAdapter {
    async fn charge(
        &self,
        _amount: Decimal,
        _currency: &str,
        _token: &str,
        idempotency_key: &str,
    ) -> Result<GatewayChargeResult, PaymentsError> {
        Ok(GatewayChargeResult {
            gateway_transaction_id: format!("manual_{}", idempotency_key),
            authorization_code: None,
            card_last_four: None,
            card_brand: None,
            raw_response: None,
        })
    }

    async fn refund(
        &self,
        gateway_tx_id: &str,
        _amount: Option<Decimal>,
        _reason: &str,
    ) -> Result<GatewayRefundResult, PaymentsError> {
        Ok(GatewayRefundResult {
            gateway_transaction_id: format!("{}_refund", gateway_tx_id),
            raw_response: None,
        })
    }

    async fn verify_webhook(
        &self,
        body: &str,
        _signature: &str,
    ) -> Result<WebhookEvent, PaymentsError> {
        // The manual adapter accepts any payload as a generic "manual.event".
        Ok(WebhookEvent {
            event_type: "manual.event".to_string(),
            gateway_transaction_id: None,
            raw_payload: body.to_string(),
        })
    }

    fn requires_manual_confirmation(&self) -> bool {
        true
    }
}
