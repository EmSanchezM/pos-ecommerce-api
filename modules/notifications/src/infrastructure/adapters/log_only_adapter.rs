//! LogOnlyAdapter — default adapter for development.
//!
//! Mirrors the role of `payments::ManualGatewayAdapter`: it never contacts an
//! external service, it just writes the notification to `tracing` so a dev
//! environment can run end-to-end without provider credentials. Production
//! deployments register a real adapter (SendGrid/Twilio/...) for the same
//! channel and override this one in the registry.

use async_trait::async_trait;

use crate::NotificationsError;
use crate::domain::entities::Notification;
use crate::domain::value_objects::NotificationChannel;

use super::notification_adapter::{DeliveryResult, NotificationAdapter};

#[derive(Debug, Clone)]
pub struct LogOnlyAdapter {
    channel: NotificationChannel,
}

impl LogOnlyAdapter {
    pub fn new(channel: NotificationChannel) -> Self {
        Self { channel }
    }
}

#[async_trait]
impl NotificationAdapter for LogOnlyAdapter {
    fn channel(&self) -> NotificationChannel {
        self.channel
    }

    async fn send(
        &self,
        notification: &Notification,
    ) -> Result<DeliveryResult, NotificationsError> {
        tracing::info!(
            id = %notification.id().into_uuid(),
            channel = %self.channel,
            recipient = notification.recipient(),
            subject = ?notification.subject(),
            "[log-only] notification dispatched"
        );
        Ok(DeliveryResult {
            provider_message_id: Some(format!("log_{}", notification.id().into_uuid())),
        })
    }
}
