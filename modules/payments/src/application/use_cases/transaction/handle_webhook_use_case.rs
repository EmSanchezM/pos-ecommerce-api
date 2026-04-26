//! HandleWebhook - signature-validates and routes a gateway webhook.
//!
//! The path segment from the route (`/api/v1/webhooks/{gateway_type}`) is used
//! to dispatch to the matching adapter through the registry. Each adapter is
//! responsible for verifying the signature against its own webhook secret.

use std::sync::Arc;

use crate::PaymentsError;
use crate::application::dtos::{WebhookPayload, WebhookResponse};
use crate::domain::repositories::TransactionRepository;
use crate::infrastructure::gateways::GatewayAdapterRegistry;

pub struct HandleWebhookUseCase {
    registry: Arc<dyn GatewayAdapterRegistry>,
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl HandleWebhookUseCase {
    pub fn new(
        registry: Arc<dyn GatewayAdapterRegistry>,
        transaction_repo: Arc<dyn TransactionRepository>,
    ) -> Self {
        Self {
            registry,
            transaction_repo,
        }
    }

    pub async fn execute(&self, payload: WebhookPayload) -> Result<WebhookResponse, PaymentsError> {
        let adapter = self.registry.for_type_str(&payload.gateway_type)?;

        let event = adapter
            .verify_webhook(&payload.raw_body, &payload.signature)
            .await?;

        // Try to correlate the event back to a stored transaction by its
        // gateway-side identifier. Unknown events are accepted but flagged.
        let transaction_id = if let Some(gateway_tx_id) = event.gateway_transaction_id.as_deref() {
            self.transaction_repo
                .find_by_gateway_transaction_id(gateway_tx_id)
                .await?
                .map(|tx| tx.id().into_uuid())
        } else {
            None
        };

        Ok(WebhookResponse {
            processed: true,
            transaction_id,
            event_type: event.event_type,
        })
    }
}
