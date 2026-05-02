//! DemandPlanningEventSubscriber — observes outbox events that change the
//! demand or stock picture (sales, goods receipts, adjustments). v1 only logs;
//! v1.1 will tag (variant, store) tuples for prioritized recompute on the next
//! tick of the nightly job.
//!
//! Mirrors the pattern in `accounting::AccountingEventSubscriber`.

use async_trait::async_trait;

use events::{EventSubscriber, EventsError, OutboxEvent};

const INTERESTED: &[&str] = &[
    "sale.completed",
    "goods_receipt.confirmed",
    "adjustment.approved",
    "stock.adjusted",
];

#[derive(Debug, Clone, Default)]
pub struct DemandPlanningEventSubscriber;

impl DemandPlanningEventSubscriber {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventSubscriber for DemandPlanningEventSubscriber {
    fn name(&self) -> &'static str {
        "demand_planning"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        INTERESTED.contains(&event_type)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        tracing::debug!(
            event_id = %event.id().into_uuid(),
            event_type = event.event_type(),
            aggregate = event.aggregate_type(),
            "[demand_planning] event observed (recompute on next nightly tick)"
        );
        Ok(())
    }
}
