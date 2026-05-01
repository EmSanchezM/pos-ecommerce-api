//! Retry job for notifications that previously hit `failed`.
//!
//! Run periodically by `api-gateway::jobs::notification_dispatcher`. Pulls a
//! batch of failed notifications below the per-record attempt cap and retries
//! them via the same adapter registry the initial send used.

use std::sync::Arc;

use crate::NotificationsError;
use crate::domain::repositories::NotificationRepository;
use crate::infrastructure::adapters::NotificationAdapterRegistry;

/// Retry attempts cap — once a notification has been tried this many times
/// it is left alone for manual review.
pub const DEFAULT_MAX_ATTEMPTS: i32 = 5;

pub struct RetryFailedNotificationsUseCase {
    repo: Arc<dyn NotificationRepository>,
    registry: Arc<dyn NotificationAdapterRegistry>,
}

impl RetryFailedNotificationsUseCase {
    pub fn new(
        repo: Arc<dyn NotificationRepository>,
        registry: Arc<dyn NotificationAdapterRegistry>,
    ) -> Self {
        Self { repo, registry }
    }

    pub async fn execute(&self, batch_size: i64) -> Result<usize, NotificationsError> {
        let mut batch = self
            .repo
            .find_retryable(DEFAULT_MAX_ATTEMPTS, batch_size)
            .await?;
        let count = batch.len();

        for notification in batch.iter_mut() {
            let adapter = self.registry.for_channel(notification.channel())?;
            match adapter.send(notification).await {
                Ok(_) => notification.mark_sent(),
                Err(err) => notification.mark_failed(err.to_string()),
            }
            if let Err(err) = self.repo.update(notification).await {
                tracing::error!(
                    id = %notification.id().into_uuid(),
                    error = %err,
                    "failed to persist notification update"
                );
            }
        }

        Ok(count)
    }
}
