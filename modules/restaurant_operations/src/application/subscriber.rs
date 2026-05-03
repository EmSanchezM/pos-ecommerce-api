//! RestaurantOperationsEventSubscriber — passive in v1.0. v1.1 listens to
//! `sale.item_added` to auto-create / extend KDS tickets routed to the right
//! station based on category/product mapping.

use async_trait::async_trait;

use events::{EventSubscriber, EventsError, OutboxEvent};

const INTERESTED: &[&str] = &[];

#[derive(Debug, Clone, Default)]
pub struct RestaurantOperationsEventSubscriber;

impl RestaurantOperationsEventSubscriber {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventSubscriber for RestaurantOperationsEventSubscriber {
    fn name(&self) -> &'static str {
        "restaurant_operations"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        INTERESTED.contains(&event_type)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        tracing::debug!(
            event_id = %event.id().into_uuid(),
            event_type = event.event_type(),
            aggregate = event.aggregate_type(),
            "[restaurant_operations] event observed (subscriber wiring lands in v1.1)"
        );
        Ok(())
    }
}
