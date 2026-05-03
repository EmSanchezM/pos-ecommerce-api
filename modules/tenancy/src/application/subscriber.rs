//! TenancyEventSubscriber — passive in v1.0. v1.1 will react to
//! `users.created` to auto-assign new users to the inviter's organization.

use async_trait::async_trait;

use events::{EventSubscriber, EventsError, OutboxEvent};

const INTERESTED: &[&str] = &[];

#[derive(Debug, Clone, Default)]
pub struct TenancyEventSubscriber;

impl TenancyEventSubscriber {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventSubscriber for TenancyEventSubscriber {
    fn name(&self) -> &'static str {
        "tenancy"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        INTERESTED.contains(&event_type)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        tracing::debug!(
            event_id = %event.id().into_uuid(),
            event_type = event.event_type(),
            aggregate = event.aggregate_type(),
            "[tenancy] event observed (subscriber wiring lands in v1.1)"
        );
        Ok(())
    }
}
