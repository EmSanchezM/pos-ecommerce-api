//! SendNotificationUseCase — persists a queued notification, hands it to the
//! adapter for the requested channel, and updates the record with the result.

use std::sync::Arc;

use crate::NotificationsError;
use crate::application::dtos::SendNotificationCommand;
use crate::domain::entities::Notification;
use crate::domain::repositories::NotificationRepository;
use crate::domain::value_objects::NotificationId;
use crate::infrastructure::adapters::NotificationAdapterRegistry;

pub struct SendNotificationUseCase {
    repo: Arc<dyn NotificationRepository>,
    registry: Arc<dyn NotificationAdapterRegistry>,
}

impl SendNotificationUseCase {
    pub fn new(
        repo: Arc<dyn NotificationRepository>,
        registry: Arc<dyn NotificationAdapterRegistry>,
    ) -> Self {
        Self { repo, registry }
    }

    pub async fn execute(
        &self,
        cmd: SendNotificationCommand,
    ) -> Result<NotificationId, NotificationsError> {
        let mut notification = Notification::create(
            cmd.channel,
            cmd.recipient,
            cmd.subject,
            cmd.body,
            cmd.metadata,
        );
        self.repo.save(&notification).await?;

        let adapter = self.registry.for_channel(cmd.channel)?;
        match adapter.send(&notification).await {
            Ok(_result) => {
                notification.mark_sent();
                self.repo.update(&notification).await?;
            }
            Err(err) => {
                notification.mark_failed(err.to_string());
                self.repo.update(&notification).await?;
                return Err(err);
            }
        }

        Ok(notification.id())
    }
}
