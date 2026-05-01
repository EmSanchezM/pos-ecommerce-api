//! AccountingEventSubscriber — observes outbox events that have accounting
//! impact (sale completed, goods receipt confirmed, payment settled,
//! adjustment approved). For now it only logs; automatic posting kicks in
//! once a chart of accounts is seeded and an `AccountMappingRule` table
//! tells the subscriber which accounts to debit/credit per source type.

use async_trait::async_trait;

use events::{EventSubscriber, EventsError, OutboxEvent};

const INTERESTED: &[&str] = &[
    "sale.completed",
    "goods_receipt.confirmed",
    "adjustment.approved",
    "payment.settled",
];

#[derive(Debug, Clone, Default)]
pub struct AccountingEventSubscriber;

impl AccountingEventSubscriber {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventSubscriber for AccountingEventSubscriber {
    fn name(&self) -> &'static str {
        "accounting"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        INTERESTED.contains(&event_type)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        tracing::debug!(
            event_id = %event.id().into_uuid(),
            event_type = event.event_type(),
            aggregate = event.aggregate_type(),
            "[accounting] event observed (auto-posting pending chart-of-accounts seed)"
        );
        Ok(())
    }
}
