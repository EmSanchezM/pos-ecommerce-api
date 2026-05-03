//! ServiceOrdersEventSubscriber — passive in v1.0. v1.1 will react to
//! `service_orders.order.created`, `.delivered`, `.canceled` plus
//! `service_orders.quote.sent` to drive `notifications` (intake confirmation,
//! quote email, ready-for-pickup, thank-you) and to invoke
//! `sales::CreateSaleUseCase` when an order is delivered.

use async_trait::async_trait;

use events::{EventSubscriber, EventsError, OutboxEvent};

const INTERESTED: &[&str] = &[];

#[derive(Debug, Clone, Default)]
pub struct ServiceOrdersEventSubscriber;

impl ServiceOrdersEventSubscriber {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventSubscriber for ServiceOrdersEventSubscriber {
    fn name(&self) -> &'static str {
        "service_orders"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        INTERESTED.contains(&event_type)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        tracing::debug!(
            event_id = %event.id().into_uuid(),
            event_type = event.event_type(),
            aggregate = event.aggregate_type(),
            "[service_orders] event observed (subscriber wiring lands in v1.1)"
        );
        Ok(())
    }
}
