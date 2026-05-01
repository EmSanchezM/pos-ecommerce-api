//! AnalyticsEventSubscriber — listens to outbox events and triggers
//! incremental updates. The actual recompute happens via the periodic job;
//! this subscriber currently logs interesting events so the upstream modules
//! can wire publishing without analytics blocking on it.
//!
//! Events of interest (subset, will grow with phases):
//!   - `sale.completed`
//!   - `goods_receipt.confirmed`
//!   - `adjustment.approved`
//!   - `payment.settled`

use async_trait::async_trait;

use events::{EventSubscriber, EventsError, OutboxEvent};

const INTERESTED: &[&str] = &[
    "sale.completed",
    "goods_receipt.confirmed",
    "adjustment.approved",
    "payment.settled",
];

#[derive(Debug, Clone, Default)]
pub struct AnalyticsEventSubscriber;

impl AnalyticsEventSubscriber {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventSubscriber for AnalyticsEventSubscriber {
    fn name(&self) -> &'static str {
        "analytics"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        INTERESTED.contains(&event_type)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        tracing::debug!(
            event_id = %event.id().into_uuid(),
            event_type = event.event_type(),
            aggregate = event.aggregate_type(),
            "[analytics] event observed (recompute on next tick)"
        );
        Ok(())
    }
}
