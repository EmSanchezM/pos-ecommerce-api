use std::sync::Arc;
use std::time::Duration;

use notifications::{
    NotificationAdapterRegistry, PgNotificationRepository, RetryFailedNotificationsUseCase,
};

/// Spawns a background task that periodically retries notifications stuck in
/// `failed` state (under the per-record attempt cap).
pub fn spawn(
    notification_repo: Arc<PgNotificationRepository>,
    registry: Arc<dyn NotificationAdapterRegistry>,
    interval_secs: u64,
    batch_size: i64,
) {
    let use_case = RetryFailedNotificationsUseCase::new(notification_repo, registry);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        interval.tick().await;

        loop {
            interval.tick().await;
            match use_case.execute(batch_size).await {
                Ok(count) => {
                    if count > 0 {
                        println!("[notification-dispatcher] retried {} notifications", count);
                    }
                }
                Err(e) => {
                    eprintln!("[notification-dispatcher] error: {}", e);
                }
            }
        }
    });
}
