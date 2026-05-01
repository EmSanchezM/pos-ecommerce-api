//! Subscriber abstraction for in-process event delivery.
//!
//! Downstream modules (analytics, notifications, accounting) implement
//! `EventSubscriber` and register themselves at startup. The dispatcher fans
//! each pending event out to every subscriber whose `interested_in` returns
//! true; a single failure does not block delivery to the others — the event is
//! retried as a whole on the next tick.

use std::sync::Arc;

use async_trait::async_trait;

use crate::EventsError;
use crate::domain::entities::OutboxEvent;

#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Stable name used in logs and error messages.
    fn name(&self) -> &'static str;

    /// True if this subscriber wants to handle events of this type
    /// (e.g. `"sale.completed"`).
    fn interested_in(&self, event_type: &str) -> bool;

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError>;
}

/// Thread-safe collection of registered subscribers built once at startup and
/// shared with the dispatcher worker.
#[derive(Clone, Default)]
pub struct SubscriberRegistry {
    subscribers: Vec<Arc<dyn EventSubscriber>>,
}

impl SubscriberRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, subscriber: Arc<dyn EventSubscriber>) {
        self.subscribers.push(subscriber);
    }

    pub fn matching(&self, event_type: &str) -> Vec<Arc<dyn EventSubscriber>> {
        self.subscribers
            .iter()
            .filter(|s| s.interested_in(event_type))
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.subscribers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.subscribers.is_empty()
    }
}
