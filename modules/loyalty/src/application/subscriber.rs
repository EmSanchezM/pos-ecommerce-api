//! LoyaltyEventSubscriber — observes `sale.completed`. v1 only logs; v1.1
//! reads the payload (sale_id, customer_id, total) and calls
//! `EarnPointsUseCase` to mint points automatically. Until publishers carry a
//! richer payload this subscriber stays passive.

use async_trait::async_trait;

use events::{EventSubscriber, EventsError, OutboxEvent};

const INTERESTED: &[&str] = &["sale.completed"];

#[derive(Debug, Clone, Default)]
pub struct LoyaltyEventSubscriber;

impl LoyaltyEventSubscriber {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventSubscriber for LoyaltyEventSubscriber {
    fn name(&self) -> &'static str {
        "loyalty"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        INTERESTED.contains(&event_type)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        tracing::debug!(
            event_id = %event.id().into_uuid(),
            event_type = event.event_type(),
            aggregate = event.aggregate_type(),
            "[loyalty] event observed (auto-earn pending v1.1)"
        );
        Ok(())
    }
}
