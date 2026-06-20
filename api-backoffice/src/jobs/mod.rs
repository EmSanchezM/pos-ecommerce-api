// Backoffice API background jobs
//
// Spawns the event_dispatcher loop with the BackofficeAuditSubscriber
// registered. This mirrors the api-gateway/src/jobs/event_dispatcher.rs
// pattern.
//
// P4-T08: Both api-gateway AND backoffice-api run the event_dispatcher so
// audit events emitted by either binary are persisted to backoffice_audit_log.

use std::sync::Arc;
use std::time::Duration;

use audit_infra::{BackofficeAuditSubscriber, PgBackofficeAuditLogRepository};
use events::{DispatchPendingEventsUseCase, PgOutboxRepository, SubscriberRegistry};
use sqlx::PgPool;

/// Spawn the event_dispatcher background job for backoffice-api.
///
/// Registers `BackofficeAuditSubscriber` so any `backoffice.audit.*` event
/// written by a handler is picked up and persisted to `backoffice_audit_log`.
pub fn spawn_event_dispatcher(pool: PgPool, interval_secs: u64, batch_size: i64) {
    let outbox_repo = Arc::new(PgOutboxRepository::new(pool.clone()));
    let audit_log_repo = Arc::new(PgBackofficeAuditLogRepository::new(pool));

    let mut registry = SubscriberRegistry::new();
    registry.register(Arc::new(BackofficeAuditSubscriber::new(audit_log_repo)));

    let use_case = DispatchPendingEventsUseCase::new(outbox_repo, registry);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        // Skip the first immediate tick
        interval.tick().await;

        loop {
            interval.tick().await;
            match use_case.execute(batch_size).await {
                Ok(count) => {
                    if count > 0 {
                        tracing::info!("[backoffice-event-dispatcher] processed {} events", count);
                    }
                }
                Err(e) => {
                    tracing::error!("[backoffice-event-dispatcher] error: {}", e);
                }
            }
        }
    });
}
