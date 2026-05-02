//! CashManagementEventSubscriber — observes `cashier_shift.closed`. v1 logs;
//! v1.1 will auto-create a pending `CashDeposit` from the shift's cash
//! totals so the manager only has to confirm + supply a deposit slip.

use async_trait::async_trait;

use events::{EventSubscriber, EventsError, OutboxEvent};

const INTERESTED: &[&str] = &["cashier_shift.closed", "payment.settled"];

#[derive(Debug, Clone, Default)]
pub struct CashManagementEventSubscriber;

impl CashManagementEventSubscriber {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventSubscriber for CashManagementEventSubscriber {
    fn name(&self) -> &'static str {
        "cash_management"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        INTERESTED.contains(&event_type)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        tracing::debug!(
            event_id = %event.id().into_uuid(),
            event_type = event.event_type(),
            aggregate = event.aggregate_type(),
            "[cash_management] event observed (auto-deposit pending v1.1)"
        );
        Ok(())
    }
}
