use std::sync::Arc;
use std::time::Duration;

use events::{DispatchPendingEventsUseCase, OutboxRepository, SubscriberRegistry};

/// Spawns a background task that periodically drains the outbox and fans
/// pending events out to in-process subscribers.
pub fn spawn(
    outbox_repo: Arc<dyn OutboxRepository>,
    subscribers: SubscriberRegistry,
    interval_secs: u64,
    batch_size: i64,
) {
    let use_case = DispatchPendingEventsUseCase::new(outbox_repo, subscribers);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        // First tick completes immediately; skip it to avoid running on startup
        interval.tick().await;

        loop {
            interval.tick().await;
            match use_case.execute(batch_size).await {
                Ok(count) => {
                    if count > 0 {
                        println!("[event-dispatcher] processed {} events", count);
                    }
                }
                Err(e) => {
                    eprintln!("[event-dispatcher] error: {}", e);
                }
            }
        }
    });
}
