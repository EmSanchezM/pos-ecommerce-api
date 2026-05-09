//! `SubscriptionsEventSubscriber` — observes `payment.confirmed` /
//! `payment.rejected` outbox events and translates them into billing-cycle
//! and dunning-attempt outcomes via `RecordPaymentOutcomeUseCase`.
//!
//! The payload contract assumed here:
//!
//! ```json
//! { "transaction_id": "<uuid>", "reason": "optional string" }
//! ```
//!
//! If the payload doesn't contain a `transaction_id`, or the id doesn't map
//! to any of our cycles/attempts, the subscriber treats it as an unrelated
//! payment (e.g. a POS sale charge) and ack's quietly.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use events::{EventSubscriber, EventsError, OutboxEvent};

use crate::application::use_cases::{PaymentOutcome, RecordPaymentOutcomeUseCase};

const INTERESTED: &[&str] = &["payment.confirmed", "payment.rejected"];

pub struct SubscriptionsEventSubscriber {
    record_outcome: Arc<RecordPaymentOutcomeUseCase>,
}

impl SubscriptionsEventSubscriber {
    pub fn new(record_outcome: Arc<RecordPaymentOutcomeUseCase>) -> Self {
        Self { record_outcome }
    }
}

#[async_trait]
impl EventSubscriber for SubscriptionsEventSubscriber {
    fn name(&self) -> &'static str {
        "subscriptions"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        INTERESTED.contains(&event_type)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        let Some(transaction_id) = extract_transaction_id(event) else {
            tracing::warn!(
                event_id = %event.id().into_uuid(),
                event_type = event.event_type(),
                "[subscriptions] payment event missing transaction_id; ignoring"
            );
            return Ok(());
        };

        let outcome = match event.event_type() {
            "payment.confirmed" => PaymentOutcome::Confirmed,
            "payment.rejected" => {
                let reason = event
                    .payload()
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("payment rejected");
                PaymentOutcome::Rejected { reason }
            }
            other => {
                tracing::debug!(
                    event_id = %event.id().into_uuid(),
                    event_type = other,
                    "[subscriptions] event not handled"
                );
                return Ok(());
            }
        };

        match self.record_outcome.execute(transaction_id, outcome).await {
            Ok(matched) => {
                if !matched {
                    tracing::debug!(
                        event_id = %event.id().into_uuid(),
                        transaction_id = %transaction_id,
                        "[subscriptions] payment is not tied to a billing cycle"
                    );
                }
                Ok(())
            }
            Err(e) => {
                // Subscribers must not crash the dispatch loop on transient
                // errors; log and retry later via the dispatcher's retry path.
                tracing::error!(
                    event_id = %event.id().into_uuid(),
                    error = %e,
                    "[subscriptions] failed to record payment outcome"
                );
                Err(EventsError::SubscriberFailed(format!(
                    "subscriptions: {}",
                    e
                )))
            }
        }
    }
}

fn extract_transaction_id(event: &OutboxEvent) -> Option<Uuid> {
    event
        .payload()
        .get("transaction_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
}
