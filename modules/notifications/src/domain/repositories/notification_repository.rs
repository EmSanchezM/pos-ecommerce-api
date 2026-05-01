//! Notification repository trait.

use async_trait::async_trait;

use crate::NotificationsError;
use crate::domain::entities::Notification;
use crate::domain::value_objects::NotificationId;

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn save(&self, notification: &Notification) -> Result<(), NotificationsError>;

    async fn update(&self, notification: &Notification) -> Result<(), NotificationsError>;

    async fn find_by_id(
        &self,
        id: NotificationId,
    ) -> Result<Option<Notification>, NotificationsError>;

    /// Returns at most `limit` notifications still in `failed` state with
    /// fewer than `max_attempts` attempts — used by the retry job.
    async fn find_retryable(
        &self,
        max_attempts: i32,
        limit: i64,
    ) -> Result<Vec<Notification>, NotificationsError>;
}
