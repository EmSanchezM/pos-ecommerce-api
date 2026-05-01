//! Dispatcher use case — drains the outbox and fans events out to subscribers.
//!
//! Run periodically by a background worker (`api-gateway::jobs::event_dispatcher`).
//! Each invocation processes up to `batch_size` events. Subscriber failures
//! are recorded on the event (`attempts`, `last_error`) but never fail the
//! batch as a whole — independent subscribers must not block one another.

use std::sync::Arc;

use crate::EventsError;
use crate::application::subscriber::SubscriberRegistry;
use crate::domain::repositories::OutboxRepository;

pub struct DispatchPendingEventsUseCase {
    repo: Arc<dyn OutboxRepository>,
    subscribers: SubscriberRegistry,
}

impl DispatchPendingEventsUseCase {
    pub fn new(repo: Arc<dyn OutboxRepository>, subscribers: SubscriberRegistry) -> Self {
        Self { repo, subscribers }
    }

    /// Process up to `batch_size` pending events. Returns how many events were
    /// drained (regardless of success/failure).
    pub async fn execute(&self, batch_size: i64) -> Result<usize, EventsError> {
        let mut events = self.repo.fetch_pending(batch_size).await?;
        let drained = events.len();

        for event in events.iter_mut() {
            let matches = self.subscribers.matching(event.event_type());
            let mut had_failure = false;
            let mut last_err: Option<String> = None;

            for sub in matches {
                if let Err(err) = sub.handle(event).await {
                    had_failure = true;
                    last_err = Some(format!("{}: {}", sub.name(), err));
                    tracing::error!(
                        event_id = %event.id().into_uuid(),
                        event_type = event.event_type(),
                        subscriber = sub.name(),
                        error = %err,
                        "subscriber failed"
                    );
                }
            }

            if had_failure {
                event.mark_attempt_failed(last_err.unwrap_or_else(|| "unknown".into()));
            } else {
                event.mark_processed();
            }

            if let Err(err) = self.repo.update_after_dispatch(event).await {
                tracing::error!(
                    event_id = %event.id().into_uuid(),
                    error = %err,
                    "failed to persist event status update"
                );
            }
        }

        Ok(drained)
    }
}
