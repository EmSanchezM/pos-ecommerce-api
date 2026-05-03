//! BookingEventSubscriber — passive in v1.0. v1.1 will react to
//! `booking.appointment.created`, `.completed`, `.canceled` to dispatch
//! confirmation/reminder/cancellation emails via the `notifications` module
//! and (for `.completed`) optionally drive `sales::CreateSaleUseCase`.

use async_trait::async_trait;

use events::{EventSubscriber, EventsError, OutboxEvent};

const INTERESTED: &[&str] = &[];

#[derive(Debug, Clone, Default)]
pub struct BookingEventSubscriber;

impl BookingEventSubscriber {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventSubscriber for BookingEventSubscriber {
    fn name(&self) -> &'static str {
        "booking"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        INTERESTED.contains(&event_type)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        tracing::debug!(
            event_id = %event.id().into_uuid(),
            event_type = event.event_type(),
            aggregate = event.aggregate_type(),
            "[booking] event observed (subscriber wiring lands in v1.1)"
        );
        Ok(())
    }
}
